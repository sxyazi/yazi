# Codebase Structure

**Analysis Date:** 2026-03-21

## Directory Layout

```
yazi/                           # Workspace root
├── Cargo.toml                  # Workspace manifest, centralized deps, clippy rules
├── Cargo.lock                  # Lockfile (committed)
├── rustfmt.toml                # Hard tabs, nightly formatting features
├── stylua.toml                 # Lua formatter config (for plugin files)
├── .cargo/config.toml          # jemalloc global allocator, linker flags
├── assets/                     # Icons, screenshots, misc assets
├── scripts/                    # Dev utilities (icon validation, form scripts)
├── nix/                        # Nix flake derivations
├── snap/                       # Snapcraft packaging
├── .github/                    # CI workflows, issue templates
├── .planning/                  # GSD planning documents (not shipped)
│
├── yazi-fm/                    # BINARY: main file manager (default workspace member)
├── yazi-cli/                   # BINARY: `ya` CLI tool (default workspace member)
│
├── yazi-actor/                 # Actor trait + all command implementations
├── yazi-adapter/               # Terminal image protocol adapters (kgp/sixel/iip/chafa/ueberzug)
├── yazi-binding/               # Lua ↔ Rust type bindings (mlua UserData)
├── yazi-boot/                  # CLI arg parsing (clap), boot configuration
├── yazi-build/                 # Build-time code generation helper binary
├── yazi-codegen/               # Proc-macro support crate
├── yazi-config/                # Config parsing (yazi.toml, keymap.toml, theme.toml)
├── yazi-core/                  # Core state structs (Core, Mgr, Tab, Tasks, overlays)
├── yazi-dds/                   # IPC pub/sub server (Unix socket, multi-instance)
├── yazi-emulator/              # Terminal emulator detection (brand, dimensions)
├── yazi-ffi/                   # macOS FFI bindings (CoreFoundation, IOKit, DiskArbitration)
├── yazi-fs/                    # Filesystem abstractions (File, Files, sorting, filtering)
├── yazi-macro/                 # Proc-macros (act!, emit!, succ!, mod_pub!, mod_flat!, render!, ...)
├── yazi-packing/               # Plugin packaging utilities
├── yazi-parser/                # Options structs for every action (parsed from ActionCow)
├── yazi-plugin/                # Lua runtime, plugin loader, isolate sandboxing
├── yazi-proxy/                 # Fire-and-forget action emitters for async contexts
├── yazi-scheduler/             # Task runner (file ops, preload, fetch, plugin, process workers)
├── yazi-sftp/                  # SFTP client (russh-based remote filesystem)
├── yazi-shared/                # Shared primitives (Event, Layer, Id, RoCell, URL types, ...)
├── yazi-shim/                  # Patches/shims for crossterm and ratatui
├── yazi-term/                  # Ratatui Terminal wrapper with partial-render support
├── yazi-tty/                   # Raw TTY handle (separate from ratatui for direct writes)
├── yazi-vfs/                   # Virtual filesystem (local + SFTP provider abstraction)
└── yazi-watcher/               # Filesystem watcher (local inotify/FSEvents + remote polling)
```

## Directory Purposes

**`yazi-fm/src/`:**
- Purpose: Binary crate for the main `yazi` process
- Contains: `main.rs`, `App` event loop, `Dispatcher`, `Router`, `Executor`, `Root` renderer, per-overlay UI widgets
- Key files: `yazi-fm/src/main.rs`, `yazi-fm/src/app/app.rs`, `yazi-fm/src/dispatcher.rs`, `yazi-fm/src/executor.rs`, `yazi-fm/src/router.rs`, `yazi-fm/src/root.rs`

**`yazi-cli/src/`:**
- Purpose: Binary crate for the `ya` companion CLI
- Contains: `main.rs`, `args.rs`, command handlers for `emit`, `emit-to`, `pub`, `pub-to`, `sub`, `pkg`
- Key files: `yazi-cli/src/main.rs`, `yazi-cli/src/package/`

**`yazi-actor/src/`:**
- Purpose: All command/action implementations; the `Actor` trait definition; `Ctx` context struct; `Lives` Lua scope manager
- Sub-directories: `app/` (15 actors), `mgr/` (~60 actors), `cmp/`, `confirm/`, `help/`, `input/`, `notify/`, `pick/`, `spot/`, `tasks/`, `which/`, `lives/`, `core/`
- Key files: `yazi-actor/src/actor.rs`, `yazi-actor/src/context.rs`, `yazi-actor/src/lives/lives.rs`

**`yazi-core/src/`:**
- Purpose: Pure state structs; no I/O, no async; mutated only by actors via `Ctx`
- Sub-directories: `mgr/` (Mgr, Tabs, Yanked, Mimetype), `tab/` (Tab, Folder, Preview, Finder, Selected, History, Backstack, Mode), `tasks/`, `cmp/`, `confirm/`, `help/`, `input/`, `notify/`, `pick/`, `spot/`, `which/`
- Key files: `yazi-core/src/core.rs`, `yazi-core/src/mgr/mgr.rs`, `yazi-core/src/tab/tab.rs`

**`yazi-config/src/`:**
- Purpose: TOML config parsing; exposes global `static` values (`YAZI`, `KEYMAP`, `THEME`, `LAYOUT`)
- Sub-directories: `keymap/`, `mgr/`, `open/`, `opener/`, `plugin/`, `popup/`, `preview/`, `tasks/`, `theme/`, `vfs/`, `which/`
- Key files: `yazi-config/src/lib.rs`, `yazi-config/preset/yazi-default.toml`, `yazi-config/preset/keymap-default.toml`

**`yazi-config/preset/`:**
- Purpose: Embedded default configurations shipped with the binary
- Contains: `yazi-default.toml`, `keymap-default.toml`, `theme-dark.toml`, `theme-light.toml`, `vfs-default.toml`

**`yazi-shared/src/`:**
- Purpose: Cross-crate primitives with no domain logic
- Contains: `event/` (Event enum, ActionCow), `url/` (UrlBuf, UrlLike, Scheme), `data/`, `errors/`, `pool/`, `strand/`, and misc types (`Id`, `Layer`, `Source`, `RoCell`, `SyncCell`, `Debounce`)
- Key files: `yazi-shared/src/event/event.rs`, `yazi-shared/src/layer.rs`

**`yazi-dds/src/`:**
- Purpose: IPC broker for multi-instance communication
- Contains: `client.rs`, `server.rs`, `pubsub.rs`, `payload.rs`, `stream.rs`, `ember/` (message frame types: Hi, Hey, Bye, payload body)
- Key files: `yazi-dds/src/client.rs`, `yazi-dds/src/server.rs`, `yazi-dds/src/lib.rs`

**`yazi-plugin/src/`:**
- Purpose: Lua 5.5 scripting engine initialization, plugin loader, coroutine/isolate sandboxing
- Sub-directories: `runtime/` (Lua globals composer), `loader/` (plugin discovery), `isolate/` (sandboxed execution), `pubsub/`, `fs/`, `external/`, `process/`, `theme/`, `utils/`
- Key files: `yazi-plugin/src/lua.rs`, `yazi-plugin/src/runtime/runtime.rs`

**`yazi-binding/src/`:**
- Purpose: Lua UserData implementations for Rust types (Url, Rect, Style, Color, File, Id, Image, etc.)
- Contains: `elements/` (ratatui widget helpers), and flat modules for each bound type
- Key files: `yazi-binding/src/elements/`, `yazi-binding/src/file.rs`, `yazi-binding/src/url.rs`

**`yazi-scheduler/src/`:**
- Purpose: Async task runner with priority queues
- Sub-directories: `fetch/`, `file/`, `hook/`, `plugin/`, `preload/`, `process/`, `size/`
- Key files: `yazi-scheduler/src/scheduler.rs`, `yazi-scheduler/src/runner.rs`

**`yazi-fs/src/`:**
- Purpose: Local filesystem abstractions: `File`, `Files` collection, URL/path helpers, sorting, filtering, XDG paths
- Contains: `file.rs`, `files.rs`, `cha.rs` (file characteristics), `sorter.rs`, `filter.rs`, `mounts.rs`, `provider/`
- Key files: `yazi-fs/src/file.rs`, `yazi-fs/src/files.rs`

**`yazi-vfs/src/`:**
- Purpose: Unified virtual filesystem provider interface; delegates to local or SFTP backend
- Contains: `provider/` with `local/`, `sftp/`; `cha.rs`, `file.rs`, `files.rs`, `op.rs`
- Key files: `yazi-vfs/src/provider/providers.rs`, `yazi-vfs/src/provider/provider.rs`

**`yazi-adapter/src/`:**
- Purpose: Image rendering to terminal; auto-selects protocol per emulator brand
- Contains: `adapter.rs`, `adapters.rs`, `drivers/` (chafa, iip, kgp, kgp_old, sixel, ueberzug), `image.rs`, `info.rs`
- Key files: `yazi-adapter/src/adapter.rs`, `yazi-adapter/src/adapters.rs`

**`yazi-macro/src/`:**
- Purpose: Proc-macros used throughout the workspace
- Contains: `actor.rs` (`act!`), `event.rs` (`emit!`, `relay!`), `render.rs` (`render!`), `module.rs` (`mod_pub!`, `mod_flat!`), `context.rs`, `fs.rs`, `fmt.rs`, `log.rs`, `stdio.rs`, `platform.rs`

**`yazi-proxy/src/`:**
- Purpose: Thin wrappers allowing async tasks to emit actions without holding `Core`
- Contains: one file per layer: `app.rs`, `mgr.rs`, `input.rs`, `cmp.rs`, `tasks.rs`, `notify.rs`, `pick.rs`, `which.rs`, `confirm.rs`

**`yazi-parser/src/`:**
- Purpose: Typed `Options` structs (`CdOpt`, `OpenOpt`, `SortOpt`, ...) deserialized from `ActionCow` parameters
- Sub-directories: one per layer matching `yazi-actor` structure

**`yazi-watcher/src/`:**
- Purpose: Watch local directory changes (platform FS events) and poll remote paths
- Sub-directories: `local/` (OS watcher backend), `remote/` (polling for VFS paths)

## Key File Locations

**Entry Points:**
- `yazi-fm/src/main.rs`: `yazi` binary entry; subsystem initialization order
- `yazi-cli/src/main.rs`: `ya` CLI entry

**Event Bus:**
- `yazi-shared/src/event/event.rs`: `Event` enum definition and global channel

**Core State:**
- `yazi-core/src/core.rs`: `Core` struct aggregating all UI component states
- `yazi-core/src/mgr/mgr.rs`: File manager state (tabs, yanked, watcher)
- `yazi-core/src/tab/tab.rs`: Per-tab state (current folder, history, selection, preview)

**Action Dispatch:**
- `yazi-fm/src/executor.rs`: Routes `ActionCow` by layer to `act!` macros
- `yazi-fm/src/router.rs`: Translates key events to action sequences via `KEYMAP`
- `yazi-actor/src/actor.rs`: `Actor` trait definition

**Actor Context:**
- `yazi-actor/src/context.rs`: `Ctx` struct — short-lived mutable `Core` borrow passed to actors

**Rendering:**
- `yazi-fm/src/root.rs`: `Root` widget — Lua-driven base + Rust overlay z-stack
- `yazi-fm/src/app/render.rs`: `App::render()` and `App::render_partially()`

**Configuration:**
- `yazi-config/src/lib.rs`: Global statics `YAZI`, `KEYMAP`, `THEME`, `LAYOUT`
- `yazi-config/preset/`: Embedded default TOML files

**IPC:**
- `yazi-dds/src/client.rs`: Client connect/reconnect loop and `shot()`/`draw()` for `ya`
- `yazi-dds/src/server.rs`: Unix socket server routing messages by ability

**Lua Integration:**
- `yazi-actor/src/lives/lives.rs`: `Lives::scope()` — pins Rust state as Lua globals
- `yazi-plugin/src/runtime/runtime.rs`: `cx` global composer (args, mgr, preview, tasks)

**Testing:**
- Tests are co-located within each crate under `src/` using `#[cfg(test)]` modules

## Naming Conventions

**Files:**
- `snake_case.rs` for all Rust source files
- `mod.rs` for module root files within subdirectories
- `lib.rs` for crate root files

**Directories:**
- `snake_case` for all source subdirectories
- Directory name matches the module name it exposes (e.g., `yazi-actor/src/mgr/` → `pub mod mgr`)

**Crates:**
- All crates prefixed with `yazi-` (hyphenated); Rust module names use `yazi_` (underscored)

**Types:**
- `PascalCase` for structs, enums, traits
- `SCREAMING_SNAKE_CASE` for module-level statics (`YAZI`, `KEYMAP`, `TX`, `RX`)
- `snake_case` for functions and variables

**Macros:**
- `snake_case!` for all proc-macros (`act!`, `emit!`, `succ!`, `render!`, `mod_pub!`)

## Where to Add New Code

**New actor (user command):**
- Implementation: `yazi-actor/src/{layer}/{command_name}.rs` implementing `Actor` trait
- Options struct: `yazi-parser/src/{layer}/{command_name}.rs` or add to existing file
- Registration: Add to `yazi-fm/src/executor.rs` in the relevant layer method using `on!(command_name)`
- Proxy helper (if needed from async): Add method to `yazi-proxy/src/{layer}.rs`

**New UI overlay:**
- State struct: `yazi-core/src/{overlay_name}/` with `mod.rs`
- Add field to `Core` in `yazi-core/src/core.rs`
- Actor implementations: `yazi-actor/src/{overlay_name}/`
- Renderer widget: `yazi-fm/src/{overlay_name}/` implementing `ratatui::Widget`
- Add `Layer` variant to `yazi-shared/src/layer.rs`
- Add executor routing in `yazi-fm/src/executor.rs`
- Register in `Core::layer()` priority chain in `yazi-core/src/core.rs`

**New configuration section:**
- Config struct: `yazi-config/src/{section}/`
- Add field to the relevant config aggregate struct
- Add default values to `yazi-config/preset/yazi-default.toml`

**New Lua binding (expose Rust type to plugins):**
- UserData impl: `yazi-binding/src/{type_name}.rs`
- Add to `yazi-actor/src/lives/` if it needs to be accessible via the `cx` global

**Shared utilities:**
- Cross-crate primitives: `yazi-shared/src/`
- Filesystem helpers: `yazi-fs/src/`
- New proc-macro: `yazi-macro/src/`

## Special Directories

**`yazi-config/preset/`:**
- Purpose: Default TOML configurations embedded into the binary at build time
- Generated: No (hand-authored)
- Committed: Yes

**`.planning/`:**
- Purpose: GSD planning documents and codebase analysis (not shipped)
- Generated: By GSD tooling
- Committed: No (gitignored or team-local)

**`target/`:**
- Purpose: Cargo build artifacts
- Generated: Yes
- Committed: No

**`assets/`:**
- Purpose: Non-code resources (icon files, screenshots)
- Generated: No
- Committed: Yes

---

*Structure analysis: 2026-03-21*
