# Default Configuration

> [!IMPORTANT]
> If you're using a stable release of Yazi instead of the newest nightly build, make sure you're checking these files out from [the `shipped` tag][shipped], not the newest `main` branch.

This directory contains the default configuration files for Yazi:

- [`yazi-default.toml`][yazi-default]: General configuration
- [`keymap-default.toml`][keymap-default]: Keybindings configuration
- [`theme-dark.toml`][theme-dark]: Dark color scheme (loaded when your terminal is in dark mode)
- [`theme-light.toml`][theme-light]: Light color scheme (loaded when your terminal is in light mode)

These files are already included with Yazi when you install the release, so you don't need to manually download or copy them to your Yazi configuration directory.

However, if you want to customize certain configurations:

- Create a `yazi.toml` in your config directory to override certain settings in [`yazi-default.toml`][yazi-default], so either:
  - `~/.config/yazi/yazi.toml` on Unix-like systems
  - `%AppData%\yazi\config\yazi.toml` on Windows
- Create a `keymap.toml` in your config directory to override certain settings in [`keymap-default.toml`][keymap-default], so either:
  - `~/.config/yazi/keymap.toml` on Unix-like systems
  - `%AppData%\yazi\config\keymap.toml` on Windows
- Create a `theme.toml` in your config directory to override certain settings in [`theme-light.toml`][theme-light] and [`theme-dark.toml`][theme-dark], so either:
  - `~/.config/yazi/theme.toml` on Unix-like systems
  - `%AppData%\yazi\config\theme.toml` on Windows

For the user's `theme.toml` file, you can only apply the same color scheme to both the light and dark themes.

If you want more granular control over colors, specify two different flavors for light and dark modes under the `[flavor]` section of your `theme.toml`, and override them in your respective flavor instead.

[shipped]: https://github.com/sxyazi/yazi/tree/shipped
[yazi-default]: yazi-default.toml
[keymap-default]: keymap-default.toml
[theme-dark]: theme-dark.toml
[theme-light]: theme-light.toml

## Learn more

- [Configuration documentation](https://yazi-rs.github.io/docs/configuration/overview)
- [Flavors documentation](https://yazi-rs.github.io/docs/flavors/overview)
