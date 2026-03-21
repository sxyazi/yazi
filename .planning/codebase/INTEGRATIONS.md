# External Integrations

**Analysis Date:** 2026-03-21

## APIs & External Services

**None (network-based APIs):**
- Yazi is a local terminal file manager. It does not call any external web APIs or cloud services at runtime.
- All "integrations" are protocol-level or subprocess-based.

## SSH / SFTP

**SSH/SFTP Protocol:**
- Implementation: `yazi-sftp/` crate
- SDK/Client: `russh` 0.57.1 (pure-Rust SSH2 implementation, ring + rsa features)
- Purpose: Browse and operate on remote filesystems over SFTP
- Session management: `yazi-sftp/src/session.rs` — async session with per-request oneshot callbacks
- Protocol: Custom SFTP packet serialization/deserialization in `yazi-sftp/src/ser.rs`, `de.rs`
- Auth: SSH key-based (RSA supported via `russh`); no env var required, handled by SSH session negotiation
- VFS integration: Remote paths handled by `typed-path` for cross-platform path normalization

## Image Display Protocols

**Terminal Image Rendering (`yazi-adapter/`):**

Yazi probes the terminal emulator and selects the best supported image protocol:

| Protocol | Driver file | Terminals |
|---|---|---|
| Kitty Graphics Protocol (KGP) | `yazi-adapter/src/drivers/kgp.rs` | Kitty, Ghostty |
| KGP Old (chunked) | `yazi-adapter/src/drivers/kgp_old.rs` | Konsole, Warp |
| iTerm2 Inline Protocol (IIP) | `yazi-adapter/src/drivers/iip.rs` | iTerm2, WezTerm, VSCode, Tabby, Hyper, Mintty, Rio, Bobcat |
| Sixel | `yazi-adapter/src/drivers/sixel.rs` | Foot, Microsoft Terminal, BlackBox, VSCode, Rio, Bobcat |
| Ueberzug++ | `yazi-adapter/src/drivers/ueberzug.rs` | Fallback via external process |
| Chafa | `yazi-adapter/src/drivers/chafa.rs` | Default fallback (text-mode rendering) |

Detection logic: `yazi-adapter/src/adapters.rs` — maps `yazi_emulator::Brand` enum to adapter list.

**ICC Color Management:**
- `moxcms` 0.8.1 — reads embedded ICC profiles from images
- `palette` 0.7.6 — color space conversion
- `quantette` 0.5.1 — color quantization for Sixel/Chafa output

## Terminal Emulator Detection

**Emulator Database (`yazi-emulator/`):**
- Detects running terminal via escape sequence queries and environment variables
- Recognized brands: Kitty, Konsole, iTerm2, WezTerm, Foot, Ghostty, Microsoft Terminal, Warp, Rio, BlackBox, VSCode, Tabby, Hyper, Mintty, Tmux, VTerm, Apple Terminal, Urxvt, Bobcat
- Light/dark theme detection feeds into `yazi-config` flavor selection
- Tmux multiplexer passthrough handled via `yazi-adapter/src/lib.rs`
- WSL detection via `yazi_shared::in_wsl()`

## Data Storage

**Databases:**
- None (no embedded database)

**File Storage:**
- Local filesystem only, via Rust standard library and `yazi-fs/` crate
- Trash/recycle bin: `trash` 5.2.5 crate (all non-Android platforms) in `yazi-fs/`
- Preview image cache: configurable directory via `preview.cache_dir` in `yazi-config/preset/yazi-default.toml`

**Caching:**
- In-process LRU cache: `lru` 0.16.3 in `yazi-scheduler/`
- No external cache service

## Plugin System

**Lua Plugin System (`yazi-plugin/`):**
- Runtime: Lua 5.5 embedded via `mlua` 0.11.6 (vendored by default)
- Prebuilt assets: `yazi-prebuilt` 0.1.0 crate bundles compiled Lua plugins
- Built-in plugins: `yazi-plugin/preset/plugins/` — archive, code, dds, image, video, pdf, svg, json, font, fzf, zoxide, mime, session, vfs, etc.
- Plugin API exposed as `ya` global in Lua; see `yazi-plugin/preset/ya.lua`
- Async plugin execution via `ya.co()` coroutine API

## Data Distribution Service (DDS)

**Inter-process Communication (`yazi-dds/`):**
- Purpose: Communication between multiple yazi instances (e.g., nesting, shell integration)
- Transport: Unix domain socket (or named pipe on Windows), using Tokio async I/O
- Protocol: JSON-serialized messages (`serde_json`)
- Environment variables set/read: `YAZI_ID`, `YAZI_PID`, `YAZI_LEVEL`
- Build embeds git metadata via `vergen-gitcl`

## Shell Integration

**CLI Tool (`yazi-cli/` → binary `ya`):**
- Shell completion generated at build time for: bash, zsh, fish, elvish, nushell, fig
- Libraries: `clap_complete` 4.6.0, `clap_complete_nushell` 4.6.0, `clap_complete_fig` 4.5.2
- `ya pub` / `ya emit` commands for DDS messaging from shell scripts

## Filesystem Watching

**File Watcher (`yazi-watcher/`):**
- Library: `notify` 8.2.0 with `macos_fsevent` feature
- macOS: FSEvents API via feature flag
- Linux: inotify (via notify default backend)
- Windows: ReadDirectoryChanges (via notify default backend)

## External Programs (Subprocess Integrations)

Yazi shells out to external programs configured in `yazi-config/preset/yazi-default.toml`. These are not hard dependencies but expected to be present in `$PATH`:

**File Operations:**
- `$EDITOR` / `vi` — text editing
- `code` — VS Code editor (Windows)
- `xdg-open` — open files (Linux)
- `open` — open files (macOS)
- `termux-open` — open files (Android)

**Media:**
- `mediainfo` — media file metadata display
- `exiftool` — EXIF data display

**Archives:**
- Handled internally via `ya pub extract` (DDS command)

**Fuzzy Finder:**
- `fzf` — optional, via `yazi-plugin/preset/plugins/fzf.lua`

**Directory Jumping:**
- `zoxide` — optional, via `yazi-plugin/preset/plugins/zoxide.lua`

**Image Fallback:**
- `ueberzug++` — external process fallback for image display
- `chafa` — text-mode image rendering fallback

**Video Preview:**
- External video thumbnailing via `yazi-plugin/preset/plugins/video.lua`

**PDF Preview:**
- External PDF rendering via `yazi-plugin/preset/plugins/pdf.lua`

**SVG Preview:**
- External SVG rendering via `yazi-plugin/preset/plugins/svg.lua`

**Font Preview:**
- `yazi-plugin/preset/plugins/font.lua`

## Authentication & Identity

**Auth Provider:**
- None (no user accounts or auth service)
- SSH auth for SFTP handled by `russh` (key-based)

## Monitoring & Observability

**Error Tracking:**
- None (no external error tracking service)

**Logging:**
- `tracing` 0.1.44 framework with `tracing-subscriber` 0.3.23
- Log appender: `tracing-appender` 0.2.4 writes to file
- Max log level: `debug` in both dev and release builds (via `max_level_debug` and `release_max_level_debug` features)
- Configurable via `RUST_LOG` environment variable (env-filter feature)

**Panic Handling:**
- `better-panic` 0.3.0 in `yazi-fm/` for improved panic display

## CI/CD & Deployment

**Hosting:**
- GitHub Releases (binary archives per platform/arch)
- Snap Store (Linux, via `snap/`)
- Winget (Windows Package Manager)
- Nix (via `flake.nix` and nixpkgs)

**CI Pipeline:**
- GitHub Actions (`.github/workflows/`)
  - `test.yml` — build + `cargo test --workspace` on Ubuntu, Windows, macOS
  - `check.yml` — clippy (stable), rustfmt (nightly), stylua
  - `publish.yml` — on release: publish to Winget and Snap Store
  - `cachix.yml` — Nix binary cache
  - `draft.yml` — release draft automation
  - `lock.yml` — dependency lock file updates
- Build cache: `sccache` via `mozilla-actions/sccache-action`

**Release Distribution:**
- Binary installs via `cargo-binstall` (metadata in `[package.metadata.binstall]`)
- Archive naming: `yazi-{target}.{archive-suffix}` from GitHub Releases

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## Environment Configuration

**Required for build:**
- Rust stable toolchain (MSRV 1.92.0)
- `MACOSX_DEPLOYMENT_TARGET=10.12` (set automatically via `.cargo/config.toml`)

**Required for CI secrets:**
- `WINGET_TOKEN` — publishing to Windows Package Manager
- `SNAPCRAFT_TOKEN` — publishing to Snap Store
- `GITHUB_TOKEN` — stylua action authentication

**Optional runtime:**
- `EDITOR` — preferred text editor (falls back to `vi`)
- `YAZI_CONFIG_HOME` — override config directory
- `RUST_LOG` — log level filter

---

*Integration audit: 2026-03-21*
