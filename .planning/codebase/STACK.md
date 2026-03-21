# Technology Stack

**Analysis Date:** 2026-03-21

## Languages

**Primary:**
- Rust 2024 edition - All core crates (`yazi-fm/`, `yazi-cli/`, `yazi-core/`, all `yazi-*` crates)

**Secondary:**
- Lua 5.5 (embedded via mlua) - Plugin system (`yazi-plugin/preset/plugins/*.lua`, `yazi-plugin/preset/ya.lua`)
- Nix - Development environment and packaging (`flake.nix`, `nix/`)
- TOML - Configuration format (`yazi-config/preset/*.toml`)

## Runtime

**Environment:**
- Native binary, no runtime required
- Platform targets: Linux, macOS, Windows, Android (via Termux)
- WSL (Windows Subsystem for Linux) supported with special detection logic

**Package Manager:**
- Cargo (Rust)
- Lockfile: `Cargo.lock` (present, committed)

## Frameworks

**Core:**
- `ratatui` 0.30.0 - TUI rendering framework (used in `yazi-fm/`, `yazi-widgets/`, `yazi-adapter/`)
- `crossterm` 0.29.0 - Cross-platform terminal I/O, with `use-dev-tty` on macOS
- `tokio` 1.50.0 (full features) - Async runtime powering the entire application

**Lua Embedding:**
- `mlua` 0.11.6 - Lua 5.5 embedding with `async`, `anyhow`, `serde`, `macros` features
- `vendored-lua` feature flag bundles Lua at compile time (default)

**Serialization:**
- `serde` 1.0.228 with derive - universal serialization
- `serde_json` 1.0.149 - JSON for DDS protocol
- `toml` 1.0.6 - Configuration file parsing
- `serde_with` 3.18.0 - Advanced serde helpers

**Build/Dev:**
- `rustfmt` nightly - Code formatting (`rustfmt.toml`)
- `clippy` stable - Linting (rules in root `Cargo.toml` `[workspace.lints.clippy]`)
- `stylua` latest - Lua code formatting (`stylua.toml`, Lua 5.4 syntax target)
- `vergen-gitcl` 9.1.0 - Build-time git metadata embedding (in `yazi-dds`, `yazi-cli` build scripts)
- `sccache` - Build caching in CI

## Key Dependencies

**Critical:**
- `mlua` 0.11.6 - Lua plugin system; entire extensibility model depends on this
- `ratatui` 0.30.0 - All UI rendering; uses unstable features `unstable-rendered-line-info`, `unstable-widget-ref`
- `tokio` 1.50.0 - All async I/O; scheduler, watcher, DDS, SFTP all depend on it
- `russh` 0.57.1 - SSH/SFTP client (ring + rsa features) used in `yazi-sftp/`

**Image Rendering:**
- `image` 0.25.10 - Image decoding (avif, bmp, dds, exr, gif, hdr, ico, jpeg, png, pnm, qoi, tga, tiff, webp) in `yazi-adapter/`
- `moxcms` 0.8.1 - ICC color management in `yazi-adapter/`
- `palette` 0.7.6 - Color manipulation in `yazi-adapter/`
- `quantette` 0.5.1 - Color quantization in `yazi-adapter/`

**Syntax Highlighting:**
- `syntect` 5.3.0 - Syntax highlighting with `plist-load` and `regex-onig` features in `yazi-plugin/`

**Filesystem:**
- `notify` 8.2.0 - Filesystem watching with `macos_fsevent` feature in `yazi-watcher/`
- `trash` 5.2.5 - Cross-platform trash/recycle bin (all non-Android targets) in `yazi-fs/`
- `typed-path` 0.12.3 - Cross-platform path handling (for SFTP and VFS)

**Performance:**
- `tikv-jemallocator` 0.6.1 - jemalloc allocator on non-macOS/non-Windows (configured in `.cargo/config.toml`)
- `parking_lot` 0.12.5 - Fast mutexes and RwLocks throughout
- `foldhash` 0.2.0 - Fast hash function
- `hashbrown` 0.16.1 - Fast hash maps
- `lru` 0.16.3 - LRU cache in `yazi-scheduler/`
- `twox-hash` 2.1.2 - xxHash3_128 for content hashing

**Utility:**
- `anyhow` 1.0.102 - Error handling throughout
- `thiserror` 2.0.18 - Error type derivation
- `clap` 4.6.0 - CLI argument parsing in `yazi-cli/`, `yazi-boot/`
- `clap_complete`, `clap_complete_nushell`, `clap_complete_fig` - Shell completion generation
- `chrono` 0.4.44 - Date/time
- `regex` 1.12.3 - Regular expressions
- `globset` 0.4.18 - Glob pattern matching for file rules
- `percent-encoding` 2.3.2 - URL encoding for VFS paths
- `base64` 0.22.1 - Encoding for image protocols
- `rand` 0.9.2 - Random number generation
- `tracing` 0.1.44 + `tracing-subscriber` 0.3.23 - Structured logging
- `tracing-appender` 0.2.4 - Log file appending
- `async-priority-channel` 0.2.0 - Priority task queue in `yazi-scheduler/`
- `yazi-prebuilt` 0.1.0 - Prebuilt Lua plugin assets bundled into `yazi-plugin/`

**Platform-specific:**
- `libc` 0.2.183 - Unix FFI (Linux, macOS)
- `uzers` 0.12.2 - Unix user/group info (Linux, macOS)
- `windows-sys` 0.61.2 - Windows API bindings (Windows; JobObjects, Storage, UI Shell)
- `core-foundation-sys` 0.8.7 - macOS CoreFoundation (macOS only)
- `objc2` 0.6.4 - Objective-C bindings (macOS only, `yazi-fs/`)
- `signal-hook-tokio` 0.4.0 - Unix signal handling in `yazi-fm/`

## Configuration

**Environment:**
- No `.env` files; configured through TOML files placed in XDG config dirs
- Runtime env vars used: `YAZI_ID`, `YAZI_LEVEL`, `YAZI_PID`, `EDITOR`, `YAZI_CONFIG_HOME`
- Build-time env via `.cargo/config.toml`: `MACOSX_DEPLOYMENT_TARGET=10.12`, jemalloc settings

**Build:**
- Root `Cargo.toml` - Workspace manifest with centralized dependency versions
- `.cargo/config.toml` - Allocator config, macOS deployment target, aarch64 CPU flags
- `rustfmt.toml` - Formatter config (nightly, hard tabs, 2-space tab width, Unix newlines)
- `stylua.toml` - Lua formatter config (Lua 5.4 syntax, 2-space indent)

**Configuration files (user-facing):**
- `yazi-config/preset/yazi-default.toml` - Manager, preview, opener, open rules
- `yazi-config/preset/keymap-default.toml` - Keybindings
- `yazi-config/preset/theme-dark.toml`, `theme-light.toml` - Color themes
- `yazi-config/preset/vfs-default.toml` - Virtual filesystem settings

## Build Profiles

**dev:** Line-tables debug info only; dependencies built without debug info
**release:** LTO=fat, codegen-units=1, panic=abort, strip=true
**release-windows:** Inherits release but panic=unwind
**dev-opt:** Release settings but 256 codegen-units, incremental=true, LTO off

## Platform Requirements

**Development:**
- Rust stable toolchain (minimum MSRV 1.92.0)
- Rust nightly for `rustfmt` formatting only
- `stylua` for Lua formatting
- Nix (optional, via `flake.nix` for reproducible dev shell)

**Production:**
- Self-contained binary; no runtime dependencies beyond system libc
- Platforms: Linux x86_64/aarch64, macOS x86_64/aarch64 (min 10.12), Windows x86_64/aarch64
- jemalloc linked statically on Linux; system allocator on macOS/Windows
- Apple M1 optimized build via `-Ctarget-cpu=apple-m1` rustflag

---

*Stack analysis: 2026-03-21*
