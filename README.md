<div align="center">
  <img src="assets/logo.png" alt="Yazi logo" width="20%">
</div>

## Yazi - ⚡️ Blazing Fast Terminal File Manager

Yazi (means "duck") is a terminal file manager written in Rust, based on non-blocking async I/O. It aims to provide an efficient, user-friendly, and customizable file management experience.

💡 A new article explaining its internal workings: [Why is Yazi Fast?](https://yazi-rs.github.io/blog/why-is-yazi-fast)

- 🚀 **Full Asynchronous Support**: All I/O operations are asynchronous, CPU tasks are spread across multiple threads, making the most of available resources.
- 💪 **Powerful Async Task Scheduling and Management**: Provides real-time progress updates, task cancellation, and internal task priority assignment.
- 🖼️ **Built-in Support for Multiple Image Protocols**: Also integrated with Überzug++, covering almost all terminals.
- 🌟 **Built-in Code Highlighting and Image Decoding**: Combined with the pre-loading mechanism, greatly accelerates image and normal file loading.
- 🔌 **Concurrent Plugin System**: UI plugins (rewriting most of the UI), functional plugins, custom previewer, and custom preloader; Just some pieces of Lua.
- 📡 **Data Distribution Service**: Built on a client-server architecture (no additional server process required), integrated with a Lua-based publish-subscribe model, achieving cross-instance communication and state persistence.
- 🧰 Integration with fd, rg, fzf, zoxide
- 💫 Vim-like input/select/which/notify component, auto-completion for cd paths
- 🏷️ Multi-Tab Support, Cross-directory selection, Scrollable Preview (for videos, PDFs, archives, directories, code, etc.)
- 🔄 Bulk Renaming, Visual Mode, File Chooser
- 🎨 Theme System, Custom Layouts, Trash Bin, CSI u
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

| Platform          | Protocol                                                                                              | Support               |
| ----------------- | ----------------------------------------------------------------------------------------------------- | --------------------- |
| kitty             | [Kitty unicode placeholders](https://sw.kovidgoyal.net/kitty/graphics-protocol/#unicode-placeholders) | ✅ Built-in           |
| Konsole           | [Kitty old protocol](https://github.com/sxyazi/yazi/blob/main/yazi-adaptor/src/kitty_old.rs)          | ✅ Built-in           |
| iTerm2            | [Inline images protocol](https://iterm2.com/documentation-images.html)                                | ✅ Built-in           |
| WezTerm           | [Inline images protocol](https://iterm2.com/documentation-images.html)                                | ✅ Built-in           |
| Mintty (Git Bash) | [Inline images protocol](https://iterm2.com/documentation-images.html)                                | ✅ Built-in           |
| foot              | [Sixel graphics format](https://www.vt100.net/docs/vt3xx-gp/chapter14.html)                           | ✅ Built-in           |
| Ghostty           | [Kitty old protocol](https://github.com/sxyazi/yazi/blob/main/yazi-adaptor/src/kitty_old.rs)          | ✅ Built-in           |
| Black Box         | [Sixel graphics format](https://www.vt100.net/docs/vt3xx-gp/chapter14.html)                           | ✅ Built-in           |
| VSCode            | [Inline images protocol](https://iterm2.com/documentation-images.html)                                | ✅ Built-in           |
| Tabby             | [Inline images protocol](https://iterm2.com/documentation-images.html)                                | ✅ Built-in           |
| Hyper             | [Inline images protocol](https://iterm2.com/documentation-images.html)                                | ✅ Built-in           |
| X11 / Wayland     | Window system protocol                                                                                | ☑️ Überzug++ required |
| Fallback          | [Chafa](https://hpjansson.org/chafa/)                                                                 | ☑️ Überzug++ required |

See https://yazi-rs.github.io/docs/image-preview for details.

## License

Yazi is MIT-licensed. For more information check the [LICENSE](LICENSE) file.
