Fonts under this directory are licensed under its own license. See the respective LICENSE for details.

| Name                                                                        | Path                                        | License                                                   |
| --------------------------------------------------------------------------- | ------------------------------------------- | --------------------------------------------------------- |
| [Computer Modern Roman](https://www.fontsquirrel.com/fonts/computer-modern) | `ComputerModern/cmunrm.ttf`                 | [LICENSE](ComputerModern/SIL%20Open%20Font%20License.txt) |
| [Noto Emoji](https://fonts.google.com/noto/specimen/Noto+Emoji)             | `NotoEmoji/NotoEmoji-VariableFont_wght.ttf` | [LICENSE](NotoEmoji/OFL.txt)                              |
| [Source Code Pro](https://fonts.google.com/specimen/Source+Code+Pro)        | `SourceCodePro/SourceCodePro-*.ttf`         | [LICENSE](SourceCodePro/OFL.txt)                          |
| [iA Writer Duo](https://github.com/iaolo/iA-Fonts/)                         | `iAWriterDuo/iAWriterDuoS-*.ttf`            | [LICENSE](iAWriterDuo/LICENSE.md)                         |

The following fonts are downloaded at build time when their respective feature is enabled:

| Name                                                                  | Feature                    | Source                                                                                                                                                                       |
| --------------------------------------------------------------------- | -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [JetBrains Mono NL](https://github.com/JetBrains/JetBrainsMono)       | `embed_jet_brains_mono_nl` | [JetBrains/JetBrainsMono](https://github.com/JetBrains/JetBrainsMono/tree/master/fonts/archives/ttf) ([OFL](https://github.com/JetBrains/JetBrainsMono/blob/master/OFL.txt)) |
| [Noto Sans JP](https://fonts.google.com/noto/specimen/Noto+Sans+JP)   | `embed_noto_sans_jp`       | [notofonts/noto-cjk](https://github.com/notofonts/noto-cjk/releases)                                                                                                         |
| [Noto Serif JP](https://fonts.google.com/noto/specimen/Noto+Serif+JP) | `embed_noto_serif_jp`      | [notofonts/noto-cjk](https://github.com/notofonts/noto-cjk/releases)                                                                                                         |
| [Recursive Sans & Mono](https://github.com/arrowtype/recursive/)      | `embed_recursive`          | [arrowtype/recursive](https://github.com/arrowtype/recursive/releases)                                                                                                       |
| [Warpnine Mono](https://github.com/0x6b/warpnine-fonts/)              | `embed_warpnine_mono`      | [0x6b/warpnine-fonts](https://github.com/0x6b/warpnine-fonts/releases)                                                                                                       |
| [Warpnine Sans](https://github.com/0x6b/warpnine-fonts/)              | `embed_warpnine_sans`      | [0x6b/warpnine-fonts](https://github.com/0x6b/warpnine-fonts/releases)                                                                                                       |

Downloaded fonts are cached in `$XDG_CACHE_HOME/typwriter/fonts` (or `~/.cache/typwriter/fonts` on macOS/Linux).
