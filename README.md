## Yazi - ⚡️ Blazing Fast Terminal File Manager

Yazi ("duck" in Chinese) is a terminal file manager written in Rust, based on non-blocking async I/O. It aims to provide an efficient, user-friendly, and configurable file management experience.

⚠️ Note: Yazi is currently in active development and may be unstable. The API is subject to change without prior notice.

## Installation

Before getting started, ensure that the following dependencies are installed on your system:

- nerd-fonts (required, for icons)
- jq (optional, for JSON preview)
- unar (optional, for archive preview)
- ffmpegthumbnailer (optional, for video thumbnails)
- fd (optional, for file searching)
- rg (optional, for file content searching)
- fzf (optional, for directory jumping)
- zoxide (optional, for directory jumping)

```bash
# Arch Linux
pacman -S ttf-nerd-fonts-symbols jq unarchiver ffmpegthumbnailer fd ripgrep fzf zoxide

# macOS
brew install jq unar ffmpegthumbnailer fd ripgrep fzf zoxide
brew tap homebrew/cask-fonts && brew install --cask font-symbols-only-nerd-font
```

Execute the following commands to clone the project and build Yazi:

```bash
git clone https://github.com/sxyazi/yazi.git
cd yazi
cargo build --release
```

## Usage

```bash
./target/release/yazi
```

## TODO

- [ ] Add example config for general usage, currently please see my [another repo](https://github.com/sxyazi/dotfiles/tree/main/yazi) instead
- [x] Integration with fzf, zoxide for fast directory navigation
- [x] Integration with fd, rg for fuzzy file searching
- [ ] Support for Überzug++ for image previews with X11/wayland environment
- [ ] Batch renaming support

## License

Yazi is MIT licensed.
