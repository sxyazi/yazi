<div align="center">
  <img src="assets/logo.png" alt="Yazi logo" width="20%">
</div>

## Yazi - ⚡️ Blazing Fast Terminal File Manager

Yazi (means "duck") is a terminal file manager written in Rust, based on non-blocking async I/O. It aims to provide an efficient, user-friendly, and customizable file management experience.

💡 A new article explaining its internal workings: [Why is Yazi Fast?](https://yazi-rs.github.io/blog/why-is-yazi-fast)

- 🚀 **Full Asynchronous Support**: All I/O operations are asynchronous, CPU tasks are spread across multiple threads, making the most of available resources.
- 💪 **Powerful Async Task Scheduling and Management**: Provides real-time progress updates, task cancellation, and internal task priority assignment.
- 🖼️ **Built-in Support for Multiple Image Protocols**: Also integrated with Überzug++ and Chafa, covering almost all terminals.
- 🌟 **Built-in Code Highlighting and Image Decoding**: Combined with the pre-loading mechanism, greatly accelerates image and normal file loading.
- 🔌 **Concurrent Plugin System**: UI plugins (rewriting most of the UI), functional plugins, custom previewer/preloader/spotter/fetcher; Just some pieces of Lua.
- 📡 **Data Distribution Service**: Built on a client-server architecture (no additional server process required), integrated with a Lua-based publish-subscribe model, achieving cross-instance communication and state persistence.
- 📦 **Package Manager**: Install plugins and themes with one command, keeping them up to date, or pin them to a specific version.
- 🧰 Integration with ripgrep, fd, fzf, zoxide
- 💫 Vim-like input/pick/confirm/which/notify component, auto-completion for cd paths
- 🏷️ Multi-Tab Support, Cross-directory selection, Scrollable Preview (for videos, PDFs, archives, code, directories, etc.)
- 🔄 Bulk Renaming, Visual Mode, File Chooser
- 🎨 Theme System, Mouse Support, Trash Bin, Custom Layouts, CSI u, OSC 52
- ... and more!

https://github.com/sxyazi/yazi/assets/17523360/92ff23fa-0cd5-4f04-b387-894c12265cc7

⚠️ Note: Yazi is currently in heavy development and may be unstable. The API is subject to change without prior notice.

## Documentation

- Usage: https://yazi-rs.github.io/docs/installation
- Features: https://yazi-rs.github.io/features

## Discussion

- Discord Server (English mainly): https://discord.gg/qfADduSdJu
- Telegram Group (Chinese mainly): https://t.me/yazi_rs

## Image Preview

| Platform                                                                    | Protocol                               | Support                                       |
| --------------------------------------------------------------------------- | -------------------------------------- | --------------------------------------------- |
| [kitty](https://github.com/kovidgoyal/kitty)                                | [Kitty unicode placeholders][kgp]      | ✅ Built-in                                   |
| [iTerm2](https://iterm2.com)                                                | [Inline images protocol][iip]          | ✅ Built-in                                   |
| [WezTerm](https://github.com/wez/wezterm)                                   | [Inline images protocol][iip]          | ✅ Built-in                                   |
| [Konsole](https://invent.kde.org/utilities/konsole)                         | [Kitty old protocol][kgp-old]          | ✅ Built-in                                   |
| [foot](https://codeberg.org/dnkl/foot)                                      | [Sixel graphics format][sixel]         | ✅ Built-in                                   |
| [Ghostty](https://mitchellh.com/ghostty)                                    | [Kitty unicode placeholders][kgp]      | ✅ Built-in                                   |
| [Windows Terminal](https://github.com/microsoft/terminal) (>= v1.22.2702.0) | [Sixel graphics format][sixel]         | ✅ Built-in                                   |
| [st with Sixel patch](https://github.com/bakkeby/st-flexipatch)             | [Sixel graphics format][sixel]         | ✅ Built-in                                   |
| [Tabby](https://github.com/Eugeny/tabby)                                    | [Inline images protocol][iip]          | ✅ Built-in                                   |
| [VSCode](https://github.com/microsoft/vscode)                               | [Inline images protocol][iip]          | ✅ Built-in                                   |
| [Rio](https://github.com/raphamorim/rio)                                    | [Inline images protocol][iip]          | ❌ Rio doesn't correctly clear images (#1786) |
| [Black Box](https://gitlab.gnome.org/raggesilver/blackbox)                  | [Sixel graphics format][sixel]         | ✅ Built-in                                   |
| [Hyper](https://github.com/vercel/hyper)                                    | [Inline images protocol][iip]          | ✅ Built-in                                   |
| X11 / Wayland                                                               | Window system protocol                 | ☑️ [Überzug++][ueberzug] required             |
| Fallback                                                                    | [ASCII art (Unicode block)][ascii-art] | ☑️ [Chafa][chafa] required                    |

See https://yazi-rs.github.io/docs/image-preview for details.

<!-- Protocols -->

[kgp]: https://sw.kovidgoyal.net/kitty/graphics-protocol/#unicode-placeholders
[kgp-old]: https://github.com/sxyazi/yazi/blob/main/yazi-adapter/src/kgp_old.rs
[iip]: https://iterm2.com/documentation-images.html
[sixel]: https://www.vt100.net/docs/vt3xx-gp/chapter14.html
[ascii-art]: https://en.wikipedia.org/wiki/ASCII_art

<!-- Dependencies -->

[ueberzug]: https://github.com/jstkdng/ueberzugpp
[chafa]: https://hpjansson.org/chafa/

## License

Yazi is MIT-licensed. For more information check the [LICENSE](LICENSE) file.
