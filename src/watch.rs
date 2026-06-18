use std::{
    error::Error,
    fmt::{Display, Formatter},
    fs::remove_file,
    future::IntoFuture,
    io::{BufRead, BufReader, stdin},
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
    thread,
};

use axum::{
    Router,
    body::Body,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::{Html, IntoResponse, Response},
    routing::get,
    serve,
};
use fs::read;
use log::{error, info};
use notify::{
    Event,
    EventKind::Modify,
    RecursiveMode, Watcher,
    event::{DataChange, ModifyKind::Data},
    recommended_watcher,
};
use open::{that_detached, with_detached};
use tokio::{
    fs,
    net::TcpListener,
    select,
    signal::ctrl_c,
    spawn,
    sync::{Notify, mpsc::unbounded_channel},
};

use crate::{CompileParams, compile};

/// Shared state for the watch server, containing server configuration and synchronization
/// primitives.
pub struct SharedState {
    /// The port number the server is listening on.
    pub port: u16,
    /// The IP address the server is bound to.
    pub address: String,
    /// Path to the input Typst file being watched.
    pub input: PathBuf,
    /// Path to the output PDF file.
    pub output: PathBuf,
    /// Notifier for signaling when the source file has changed.
    pub changed: Notify,
    /// Notifier for signaling server shutdown.
    pub shutdown: Notify,
    /// The PDF fitting type for browser display.
    pub fitting_type: FittingType,
}

// list of supported extensions
const EXTENSIONS: [&str; 16] = [
    "cbor", "csv", "gif", "htm", "html", "jpeg", "jpg", "json", "png", "svg", "toml", "txt", "typ",
    "xml", "yaml", "yml",
];

/// Starts a web server that serves the output PDF file, while watching for changes in the input
/// Typst file and recompiles when a change is detected.
///
/// Changes to `typ` files, along with files with extensions `cbor`, `csv`, `gif`, `htm`, `html`,
/// `jpeg`, `jpg`, `json`, `png`, `svg`, `toml`, `txt`, `xml`, `yaml`, and `yml` in the same
/// directory, recursively, will be watched. This is inspired by [ItsEthra/typst-live](https://github.com/ItsEthra/typst-live/).
///
/// # Arguments
///
/// - `params` - [`CompileParams`] struct.
/// - `open` - Whether to open the output PDF file with the default browser once after the server
///   launches.
/// - `app` - Open the output PDF file with the given application
///
/// # Example
///
/// Following is an example of how to use the `watch` function:
///
///```no_run
/// let rt = tokio::runtime::Runtime::new().unwrap();
/// let params = typwriter::CompileParams {
///     input: std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
///         .join("examples")
///         .join("sample.typ"),
///     output: std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
///         .join("examples")
///         .join("sample.pdf"),
///     font_paths: vec!["assets".into()],
///     dict: vec![("input".to_string(), "value".to_string())],
///     ppi: None,
///     package_path: None,
///     package_cache_path: None,
///     pdf_standards: None,
/// };
///
/// rt.block_on(async {
///     if let Err(error) = typwriter::watch(&params, true, None, Some(typwriter::FittingType::Width)).await {
///         eprintln!("Server error: {}", error)
///     }
/// });
/// ```
pub async fn watch(
    params: &CompileParams,
    open: bool,
    app: Option<&str>,
    fitting_type: Option<FittingType>,
) -> Result<(), Box<dyn Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = TcpListener::bind(&addr).await?;
    let address = listener.local_addr()?.ip().to_string();
    let port = listener.local_addr()?.port();

    let input = params.input.clone();
    let output = params.output.clone();
    let params = params.clone();

    match compile(&params) {
        Ok(duration) => {
            info!("Initial compilation succeeded in {duration:?}. Watching for changes...");
        }
        Err(why) => error!("{why}"),
    }

    let state = Arc::new(SharedState {
        port,
        address,
        input: input.clone(),
        output,
        changed: Notify::new(),
        shutdown: Notify::new(),
        fitting_type: fitting_type.unwrap_or_default(),
    });
    let state_handler = Arc::clone(&state);
    let state_selector = Arc::clone(&state);

    let router = Router::new()
        .route("/", get(root))
        .route("/target.pdf", get(pdf))
        .route("/listen", get(listen))
        .with_state(Arc::clone(&state));
    info!("Listening on {}:{}", state.address, state.port);

    if open {
        if let Some(app) = app {
            match with_detached(format!("http://{}:{}", state.address, state.port), app) {
                Ok(()) => info!("Opened in default browser"),
                Err(why) => error!("{why}"),
            }
        } else {
            match that_detached(format!("http://{}:{}", state.address, state.port)) {
                Ok(()) => info!("Opened in default browser"),
                Err(why) => error!("{why}"),
            }
        }
    }

    spawn(async move {
        info!("Press Enter to recompile, Ctrl+C to exit");
        async {
            ctrl_c().await.expect("Failed to register handler for Ctrl+C");
        }
        .await;
        state_handler.shutdown.notify_waiters();
    });

    let (stdin_tx, mut stdin_rx) = unbounded_channel();
    thread::spawn(move || {
        let stdin = stdin();
        let reader = BufReader::new(stdin.lock());
        for _ in reader.lines().map_while(Result::ok) {
            if stdin_tx.send(()).is_err() {
                break;
            }
        }
    });

    let params_stdin = params.clone();
    let state_stdin = Arc::clone(&state);
    spawn(async move {
        loop {
            select! {
                result = stdin_rx.recv() => {
                    if result.is_none() {
                        break;
                    }
                    info!("Manual recompilation triggered");
                    match compile(&params_stdin) {
                        Ok(duration) => info!("compilation succeeded in {duration:?}"),
                        Err(why) => error!("{why}"),
                    }
                    state_stdin.changed.notify_one();
                }
                () = state_stdin.shutdown.notified() => {
                    break;
                }
            }
        }
    });

    let mut watcher = recommended_watcher(move |res: Result<Event, _>| match res {
        Ok(event) => {
            if let Modify(Data(DataChange::Content)) = event.kind {
                let changed = !event
                    .paths
                    .iter()
                    .filter_map(|p| p.extension())
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .filter(|e| EXTENSIONS.contains(&e.as_str()))
                    .collect::<Vec<_>>()
                    .is_empty();
                if !changed {
                    return;
                }
                info!("Change detected. Recompiling...");
                match compile(&params) {
                    Ok(duration) => info!("compilation succeeded in {duration:?}"),
                    Err(why) => error!("{why}"),
                }
                state.changed.notify_one();
            }
        }
        Err(e) => error!("watch error: {e:?}"),
    })?;
    watcher.watch(input.parent().unwrap(), RecursiveMode::Recursive)?;
    let server = serve(listener, router).into_future();

    select! {
        _ = server => {}
        () = state_selector.shutdown.notified() => {
            info!("Shutting down...");
            watcher.unwatch(input.parent().unwrap())?;
            remove_file(&state_selector.output)?;
        }
    }

    info!("Bye!");
    Ok(())
}

/// Serves the root HTML page that embeds the PDF viewer.
pub async fn root(State(state): State<Arc<SharedState>>) -> Html<String> {
    include_str!("../assets/index.html")
        .replace("{addr}", &state.address)
        .replace("{port}", &state.port.to_string())
        .replace("{input}", &state.input.display().to_string())
        .replace("{fitting_type}", &state.fitting_type.to_string())
        .into()
}

/// Serves the compiled PDF file.
pub async fn pdf(State(state): State<Arc<SharedState>>) -> impl IntoResponse {
    Response::builder()
        .header("Content-Type", "application/pdf")
        .body(Body::from(match read(&state.output).await {
            Ok(data) => data,
            Err(why) => panic!("{why:#?}"),
        }))
        .unwrap()
}

/// WebSocket endpoint that notifies clients when the PDF is recompiled.
pub async fn listen(
    State(state): State<Arc<SharedState>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handler(socket, state))
}

async fn handler(mut socket: WebSocket, state: Arc<SharedState>) {
    loop {
        state.changed.notified().await;
        _ = socket.send(Message::Text("refresh".into())).await;
    }
}

/// Fitting type for the PDF viewer.
///
/// These values correspond to Chrome's PDF viewer URL parameters.
/// See [Chrome PDF viewer source](https://chromium.googlesource.com/chromium/src/+/6363f8da6aae63abedc87f60b629585f10bd8940/chrome/browser/resources/pdf/open_pdf_params_parser.js#61).
#[derive(Debug, Clone, Default)]
pub enum FittingType {
    /// Fit the entire page in the viewport.
    Page,
    /// Fit the page width to the viewport width.
    #[default]
    Width,
    /// Fit the page height to the viewport height.
    Height,
}

impl From<&str> for FittingType {
    fn from(value: &str) -> Self {
        match value {
            "width" => FittingType::Width,
            "height" => FittingType::Height,
            _ => FittingType::Page,
        }
    }
}

impl Display for FittingType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // https://chromium.googlesource.com/chromium/src/+/6363f8da6aae63abedc87f60b629585f10bd8940/chrome/browser/resources/pdf/open_pdf_params_parser.js#61
        match self {
            FittingType::Page => write!(f, "fit"),
            FittingType::Width => write!(f, "fith"),
            FittingType::Height => write!(f, "fitv"),
        }
    }
}
