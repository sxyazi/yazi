## Yazi - ⚡️ Blazing Fast Terminal File Manager

Yazi ("duck" in Chinese) is a terminal file manager written in Rust, based on non-blocking async I/O. It aims to provide an efficient, user-friendly, and customizable file management experience.

https://github.com/sxyazi/yazi/assets/17523360/740a41f4-3d24-4287-952c-3aec51520a32

⚠️ Note: Yazi is currently in active development and may be unstable. The API is subject to change without prior notice.

## Prerequisites

- nerd-fonts ([_optional_](https://github.com/sxyazi/yazi/wiki/I-don't-like-nerd%E2%80%90fonts!))
- ffmpegthumbnailer (_optional_, for video thumbnails)
- unar (_optional_, for archive preview)
- jq (_optional_, for JSON preview)
- poppler (_optional_, for PDF preview)
- fd (_optional_, for file searching)
- rg (_optional_, for file content searching)
- fzf (_optional_, for directory jumping)
- zoxide (_optional_, for directory jumping)

## Installation

<details>

<summary>Arch Linux</summary>

Install with paru or your favorite AUR helper:

```bash
paru -S yazi ffmpegthumbnailer unarchiver jq poppler fd ripgrep fzf zoxide
```

Or, you can replace `yazi` with `yazi-bin` package if you want pre-built binary instead of compiling by yourself.

</details>

<details>

<summary>macOS</summary>

Install Yazi and its dependencies with Homebrew:

```bash
brew install yazi ffmpegthumbnailer unar jq poppler fd ripgrep fzf zoxide
brew tap homebrew/cask-fonts && brew install --cask font-symbols-only-nerd-font
```

If you prefer to use the most recent code, use `--HEAD` flag:

```bash
brew install yazi --HEAD
```

Or you can install Yazi via cargo:

```bash
cargo install --git https://github.com/sxyazi/yazi.git
```

</details>

<details>

<summary>Nix</summary>

The [Nix package of Yazi](https://search.nixos.org/packages?channel=unstable&show=yazi) is available. Nix users can install Yazi via:

```bash
# On NixOS:
nix-env -iA nixos.yazi

# On Non NixOS:
nix-env -iA nixpkgs.yazi
```

Or add the following to your configuration:

```nix
# configuration.nix
environment.systemPackages = with pkgs; [
	yazi
];
```

You can also manage Yazi's configuration using the
[home-manager](https://nix-community.github.io/home-manager/options.html#opt-programs.yazi.enable).

</details>

<details>

<summary>Windows</summary>

See [Windows Installation Guide](https://github.com/sxyazi/yazi/wiki/Windows-Installation-Guide).

</details>

<details>

<summary>Build from source</summary>

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

</details>

## Usage

```bash
yazi
```

There is a wrapper of yazi, that provides the ability to change the current working directory when yazi exiting, feel free to use it:

```bash
function ya() {
	tmp="$(mktemp -t "yazi-cwd.XXXXX")"
	yazi --cwd-file="$tmp"
	if cwd="$(cat -- "$tmp")" && [ -n "$cwd" ] && [ "$cwd" != "$PWD" ]; then
		cd -- "$cwd"
	fi
	rm -f -- "$tmp"
}
```

## Configuration

If you want to use your own config, copy the [config folder](./config/preset) to `~/.config/yazi`, and modify it as you like.

[The documentation of all available options](./config/docs)

## Discussion

- Discord Server (English mainly): https://discord.gg/qfADduSdJu
- Telegram Group (Chinese mainly): https://t.me/yazi_rs

## Image Preview

| Platform          | Protocol                                                                         | Support               |
| ----------------- | -------------------------------------------------------------------------------- | --------------------- |
| Kitty             | [Terminal graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/) | ✅ Built-in           |
| WezTerm           | [Terminal graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/) | ✅ Built-in           |
| Konsole           | [Terminal graphics protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/) | ✅ Built-in           |
| iTerm2            | [Inline images protocol](https://iterm2.com/documentation-images.html)           | ✅ Built-in           |
| Mintty (Git Bash) | [Inline images protocol](https://iterm2.com/documentation-images.html)           | ✅ Built-in           |
| Hyper             | [Sixel graphics format](https://www.vt100.net/docs/vt3xx-gp/chapter14.html)      | ✅ Built-in           |
| foot              | [Sixel graphics format](https://www.vt100.net/docs/vt3xx-gp/chapter14.html)      | ✅ Built-in           |
| Black Box         | [Sixel graphics format](https://www.vt100.net/docs/vt3xx-gp/chapter14.html)      | ✅ Built-in           |
| X11 / Wayland     | Window system protocol                                                           | ☑️ Überzug++ required |
| Fallback          | [Chafa](https://hpjansson.org/chafa/)                                            | ☑️ Überzug++ required |

Yazi automatically selects the appropriate preview method for you, based on the priority from top to bottom.
That's relying on the `$TERM`, `$TERM_PROGRAM`, and `$XDG_SESSION_TYPE` variables, make sure you don't overwrite them by mistake!

For instance, if your terminal is Alacritty, which doesn't support displaying images itself, but you are running on an X11/Wayland environment,
it will automatically use the "Window system protocol" to display images -- this requires you to have [Überzug++](https://github.com/jstkdng/ueberzugpp) installed.

## TODO

See [Feature requests](https://github.com/sxyazi/yazi/issues/51) for more details.

## License

Yazi is MIT licensed.
