## Yazi - ⚡️ Blazing Fast Terminal File Manager

Yazi ("duck" in Chinese) is a terminal file manager written in Rust, based on non-blocking async I/O. It aims to provide an efficient, user-friendly, and configurable file management experience.

https://github.com/sxyazi/yazi/assets/17523360/740a41f4-3d24-4287-952c-3aec51520a32

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

### Arch Linux

Install with paru or your favorite AUR helper:

```bash
paru -S yazi jq unarchiver ffmpegthumbnailer fd ripgrep fzf zoxide
```

### macOS

Install the dependencies with Homebrew:

```bash
brew install jq unar ffmpegthumbnailer fd ripgrep fzf zoxide
brew tap homebrew/cask-fonts && brew install --cask font-symbols-only-nerd-font
```

And download the latest release [from here](https://github.com/sxyazi/yazi/releases). Or you can install Yazi with cargo:

```bash
cargo install --git https://github.com/sxyazi/yazi.git
```

### Build from source

Execute the following commands to clone the project and build Yazi:

```bash
git clone https://github.com/sxyazi/yazi.git
cd yazi
cargo build --release
```

Then, you can run:

```bash
./target/release/yazi
```

## Usage

```bash
yazi
```

If you want to use your own config, copy the [config folder](https://github.com/sxyazi/yazi/tree/main/config) to `~/.config/yazi`, and modify it as you like.

## TODO

- [x] Add example config for general usage, currently please see my [another repo](https://github.com/sxyazi/dotfiles/tree/main/yazi) instead
- [x] Integration with fzf, zoxide for fast directory navigation
- [x] Integration with fd, rg for fuzzy file searching
- [ ] Documentation of commands and options
- [ ] Support for Überzug++ for image previews with X11/wayland environment
- [ ] Batch renaming support

## License

Yazi is MIT licensed.
