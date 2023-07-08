## Yazi - ⚡️ Blazing Fast Terminal File Manager

Yazi ("duck" in Chinese) is a terminal file manager written in Rust, based on non-blocking async I/O. It aims to provide an efficient, user-friendly, and configurable file management experience.

⚠️ Note: Yazi is currently in active development and may be unstable. The API is subject to change without prior notice. Please use it with caution in a non-production environment.

## Installation

Before getting started, ensure that the following dependencies are installed on your system:

- jq (optional, for JSON preview)
- ffmpegthumbnailer (optional, for video thumbnails)
- fzf (optional, for fuzzy search)
- rg (optional, for fuzzy search)
- zoxide (optional, for directory jumping)

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

- Add example config for general usage, currently please see my [another repo](https://github.com/sxyazi/dotfiles/tree/main/yazi) instead
- Integration with zoxide for fast directory navigation
- Integration with fzf, rg for fuzzy file searching
- Support for Überzug++ for image previews with X11/wayland environment
- Batch renaming support

## License

Yazi is MIT licensed.
