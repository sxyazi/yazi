# Yazi Default Configuration

This directory contains the default configuration files for Yazi:

- [`yazi-default.toml`][yazi-default]: General configuration
- [`keymap-default.toml`][keymap-default]: Keybindings configuration
- [`theme-dark.toml`][theme-dark]: Dark color scheme configuration (loaded when your terminal is in dark mode)
- [`theme-light.toml`][theme-light]: Light color scheme configuration (loaded when your terminal is in light mode)

These files are already included with Yazi when you install the release, so you don't need to manually download or copy them to your Yazi configuration directory.

However, if you want to customize certain configurations:

- Create a `yazi.toml` in your config directory to override the settings in [`yazi-default.toml`][yazi-default], so either:
  - `~/.config/yazi/yazi.toml` on Unix-like systems
  - `C:\Users\USERNAME\AppData\Roaming\yazi\config\yazi.toml` on Windows
- Create a `keymap.toml` in your config directory to override the settings in [`keymap-default.toml`][keymap-default], so either:
  - `~/.config/yazi/keymap.toml` on Unix-like systems
  - `C:\Users\USERNAME\AppData\Roaming\yazi\config\keymap.toml` on Windows
- Create a `theme.toml` in your config directory to override the settings in [`theme-light.toml`][theme-light] and [`theme-dark.toml`][theme-dark], so either:
  - `~/.config/yazi/theme.toml` on Unix-like systems
  - `C:\Users\USERNAME\AppData\Roaming\yazi\config\theme.toml` on Windows

For the user's `theme.toml` file, you can only apply the same color scheme to both the light and dark themes.

If you want more granular control over colors, specify two different flavors for light and dark modes under the `[flavor]` section of your `theme.toml`, and override them there instead.

[yazi-default]: yazi-default.toml
[keymap-default]: keymap-default.toml
[theme-dark]: theme-dark.toml
[theme-light]: theme-light.toml
