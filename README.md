## Yazi - ⚡️ Blazing Fast Terminal File Manager

Yazi ("duck" in Chinese) is a terminal file manager written in Rust, based on non-blocking async I/O. It aims to provide an efficient, user-friendly, and customizable file management experience.

💫 A new article explaining its internal workings: [Why Yazi Fast?](https://github.com/sxyazi/yazi/issues/143)

- 🚀 **Full Asynchronous Support**: All I/O operations are asynchronous, CPU tasks are spread across multiple threads, making the most of available resources.
- 💪 **Powerful Async Task Scheduling and Management**: Provides real-time progress updates, task cancellation, and internal task priority assignment.
- 🖼️ **Built-in Support for Multiple Image Protocols**: Also integrated with Überzug++, covering almost all terminals.
- 🌟 **Built-in Code Highlighting and Image Encoding**: Combined with the pre-caching mechanism, greatly accelerates image and normal file loading.
- 🧰 Integration with fd, rg, fzf, zoxide
- 💫 Vim-like Input component, and Select component
- 🏷️ Multi-Tab Support, Scrollable Preview (for videos, PDFs, archives, directories, code, etc.)
- 🔄 Batch Renaming, Visual Mode, File Chooser
- 🎨 Theme System, Custom Layouts, Trash Bin, CSI u
- ... and more!

https://github.com/sxyazi/yazi/assets/17523360/d5d8427b-e0f3-4552-ae1a-553ba1a7d57e

⚠️ Note: Yazi is currently in active development and may be unstable. The API is subject to change without prior notice.

## Table of contents

* [Prerequisites](#prerequisites)
* [Installation](#installation)
* [Usage](#usage)
	* [Navigation](#navigation)
	* [Selection](#selection)
	* [File/directory operations](#filedirectory-operations)
	* [Copying paths](#copying-paths)
	* [Finding files/directories](#finding-filesdirectories)
	* [Sorting](#sorting)
	* [Further usage](#further-usage)
* [Changing working directory when exiting yazi](#changing-working-directory-when-exiting-yazi)
* [Configuration](#configuration)
* [Discussion](#discussion)
* [Image Preview](#image-preview)
* [TODO](#todo)
* [License](#license)

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

Install Yazi from [AUR](https://aur.archlinux.org/packages/yazi/) or [Arch Linux CN](https://github.com/archlinuxcn/repo/):

```bash
paru -S yazi ffmpegthumbnailer unarchiver jq poppler fd ripgrep fzf zoxide
```

You can install `yazi-bin` from [AUR](https://aur.archlinux.org/packages/yazi/) if you perfer pre-built binaries:

```bash
paru -S yazi-bin ffmpegthumbnailer unarchiver jq poppler fd ripgrep fzf zoxide
```

If you want to use the latest git version, you can install with the following command:

```bash
paru -S yazi-git ffmpegthumbnailer unarchiver jq poppler fd ripgrep fzf zoxide
```

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

To open yazi simply run the `yazi` command in your terminal:

```bash
yazi
```

To quit press the `q` key and press `~` to open the help menu.

### Navigation

To navigate between files and directories you can use the arrow keys `←`, `↑`, `↓` and `→` or vi-like commands such as `h`, `j`, `k`, `l` as following:

| Key binding | Alternate key | Action |
| ----------- | ------------- | ------ |
| ↑ | j | Move the cursor up |
| ↓ | k | Move the cursor down |
| → | l | Change into highlighted directory |
| ← | h | Change into parent directory |

Further navigation commands can be found in the table below.

| Key binding | Action |
| ----------- | ------ |
| K | Move the cursor up 5 lines |
| J | Move the cursor down 5 lines |
| g | Move cursor to the top |
| G | Move cursor to the bottom |

### Selection

To select files and directories, the following commands are available.

| Key binding | Action |
| ----------- | ------ |
| \<Space\> | Toggle selection of highlighted file/directory |
| v | Enter visual mode (selection mode) |
| V | Enter visual mode (unset mode) |
| \<Ctrl+a\> | Select all files |
| \<Ctrl+r\> | Inverse selection of all files |
| \<Esc\> | Cancel selection |

### File/directory operations

To interact with selected files/directories use any of the commands below.

| Key binding | Action |
| ----------- | ------ |
| o | Open the selected files |
| O | Open the selected files interactively |
| \<Enter\> | Open the selected files |
| \<Ctrl+Enter\> | Open the selected files interactively |  # It's cool if you're using a terminal that supports CSI u
| y | Copy the selected files |
| x | Cut the selected files |
| p | Paste the files |
| P | Paste the files (overwrite if the destination exists) |
| k | Paste the files (follow the symlinks) |
| K | Paste the files (overwrite + follow) |
| d | Move the files to the trash |
| D | Permanently delete the files |
| a | Create a file or directory (end with "/" for directories) |
| r | Rename a file or directory |
| ; | Run a shell command |
| : | Run a shell command (block the UI until the command finishes) |
| . | Toggle the visibility of hidden files |
| s | Search files by name using fd |
| S | Search files by content using ripgrep |
| \<Ctrl+s\> | Cancel the ongoing search |
| z | Jump to a directory using zoxide |
| Z | Jump to a directory, or reveal a file using fzf |

### Copying paths

To copy paths, use any of the following commands below.

*Observation: `c ⇒ d` indicates pressing the `c` key followed by pressing the `d` key.*

| Key binding | Action |
| ----------- | ------ |
| c ⇒ c | Copy absolute path |
| c ⇒ d | Copy the path of the parent directory |
| c ⇒ f | Copy the name of the file |
| c ⇒ n | Copy the name of the file without the extension |


### Finding files/directories

| Key binding | Action |
| ----------- | ------ |
| / | Forward search file/dir in current directory |
| ? | Backward search file/dir in current director |
| - | Jump to next occurrence |
| = | Jump to previous  occurrence |

### Sorting

To sort files/directories use the following commands.

*Observation: `, ⇒ a` indicates pressing the `,` key followed by pressing the `a` key.*


| Key binding | Action |
| ----------- | ------ |
| , ⇒ a | Sort alphabetically, directories first |
| , ⇒ A | Sort alphabetically, directories first (reverse) |
| , ⇒ c | Sort by creation time, directories first |
| , ⇒ C | Sort by creation time, directories first (reverse) |
| , ⇒ m | Sort by modified time, directories first |
| , ⇒ M | Sort by modified time, directories first (reverse) |
| , ⇒ n | Sort naturally, directories first |
| , ⇒ N | Sort naturally, directories first (reverse) |
| , ⇒ s | Sort by size, directories first |
| , ⇒ S | Sort by size, directories first (reverse) |


### Further usage

To see all key bindings, check the [config/preset/keymap.toml](config/preset/keymap.toml) file.

## Changing working directory when exiting yazi

There is a wrapper of yazi, that provides the ability to change the current working directory when exiting yazi, feel free to use it:

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

Here is a guide for tmux users: [Image preview within tmux](https://github.com/sxyazi/yazi/wiki/Image-preview-within-tmux)

## TODO

See [Feature requests](https://github.com/sxyazi/yazi/issues/51) for more details.

## License

Yazi is MIT licensed.
