# Architecture

**Analysis Date:** 2026-03-21

## Pattern Overview

**Overall:** Actor-based event-driven TUI application with layered state management

**Key Characteristics:**
- Async event loop (Tokio) consuming terminal input events and dispatching to named `Actor` implementations
- Strict separation between state (`yazi-core`), actions (`yazi-actor`), and rendering (`yazi-fm` UI widgets)
- Lua scripting layer (`yazi-plugin`) hooks into the same state via `Lives` scoped userdata
- Cross-process pub/sub via a Unix socket server (`yazi-dds`) allowing multiple yazi instances and the `ya` CLI to communicate

## Layers

**Entry Point / Binary:**
- Purpose: Initialize all subsystems, start the Tokio event loop, drive `App::serve()`
- Location: `yazi-fm/src/main.rs`
- Contains: `main()`, subsystem `init()` calls in order, global allocator (jemalloc)
- Depends on: Every other crate
- Used by: OS process runner

**App Loop (`App`):**
- Purpose: Receive `Event` variants from a Tokio MPSC channel; throttle renders (max 10ms interval); coordinate `Dispatcher`
- Location: `yazi-fm/src/app/app.rs`
- Contains: `Core`, `Term`, `Signals`; the `serve()` async loop
- Depends on: `yazi-core`, `yazi-term`, `yazi-actor`, `yazi-shared`
- Used by: `main()`

**Dispatcher:**
- Purpose: Match each `Event` variant to the right handler — key events go to `Router`, action calls go to `Executor`, render/resize/mouse/paste handled directly
- Location: `yazi-fm/src/dispatcher.rs`
- Contains: `dispatch()` method with exhaustive `Event` match
- Depends on: `Router`, `Executor`, `yazi-actor::Ctx`
- Used by: `App::serve()`

**Router:**
- Purpose: Translate a `Key` press into an `ActionCow` sequence by looking up `KEYMAP` for the current `Layer`
- Location: `yazi-fm/src/router.rs`
- Contains: `route()`, `matches()`; multi-key chords activate the `which` overlay
- Depends on: `yazi-config::KEYMAP`, `yazi-actor::Ctx`
- Used by: `Dispatcher::dispatch_key()`

**Executor:**
- Purpose: Dispatch a named `ActionCow` to the correct `Actor::act()` call, organized by `Layer`
- Location: `yazi-fm/src/executor.rs`
- Contains: `execute()` with per-layer handler methods (`app`, `mgr`, `tasks`, `spot`, `pick`, `input`, `confirm`, `help`, `cmp`, `which`, `notify`)
- Depends on: `yazi-actor`, `yazi-macro::act!`
- Used by: `Dispatcher::dispatch_call()`

**Core State (`Core`):**
- Purpose: Owns all live application state as a single flat struct
- Location: `yazi-core/src/core.rs`
- Contains: `Mgr`, `Tasks`, `Pick`, `Input`, `Confirm`, `Help`, `Cmp`, `Which`, `Notify`
- Depends on: `yazi-core` sub-modules
- Used by: `App`, `Ctx`, `Root` renderer

**Mgr / Tab:**
- Purpose: File manager state — multiple tabs, yanked clipboard, filesystem watcher, MIME cache
- Location: `yazi-core/src/mgr/mgr.rs`, `yazi-core/src/tab/tab.rs`
- Contains: `Mgr { tabs, yanked, watcher, mimetype }`, `Tab { current, parent, history, selected, spot, preview, finder, search, ... }`
- Depends on: `yazi-fs`, `yazi-watcher`, `yazi-vfs`
- Used by: `Core`, `Ctx`

**Actor Trait & Implementations:**
- Purpose: Each user-visible command is a zero-size struct implementing `Actor { type Options; fn act(cx, opt) -> Result<Data> }`
- Location: `yazi-actor/src/actor.rs` (trait), `yazi-actor/src/mgr/`, `yazi-actor/src/app/`, etc.
- Contains: ~60 mgr actors (cd, arrow, open, yank, paste, search, tab_create, upload...), app actors (bootstrap, resize, plugin, quit...), plus per-overlay actors
- Depends on: `yazi-actor::Ctx`, `yazi-proxy`, `yazi-parser`, `yazi-macro::act!`
- Used by: `Executor` via the `act!` macro

**Ctx (Action Context):**
- Purpose: Borrows `Core` mutably for the duration of an action; provides convenience accessors for the active tab, current folder, hovered file
- Location: `yazi-actor/src/context.rs`
- Contains: `Ctx { core, term, tab, level, source }`; `Deref`/`DerefMut` into `Core`
- Depends on: `yazi-core::Core`, `yazi-term::Term`
- Used by: every `Actor::act()` implementation

**Proxy Layer:**
- Purpose: Allow background async tasks (scheduler workers, plugin coroutines) to emit actions without holding `Core` references — they emit `Event::Call` via the shared channel
- Location: `yazi-proxy/src/`
- Contains: `MgrProxy`, `InputProxy`, `CmpProxy`, `TasksProxy`, etc. — thin wrappers that call `emit!(Call(relay!(...)))`
- Depends on: `yazi-shared::event::Event`, `yazi-macro`
- Used by: `yazi-actor` async tasks, `yazi-scheduler`, `yazi-plugin`

**Parser Layer:**
- Purpose: Typed `Options` structs for every action; decouples serialization/deserialization from actor logic
- Location: `yazi-parser/src/`
- Contains: Per-overlay modules (`mgr`, `app`, `input`, ...) with structs like `CdOpt`, `OpenOpt`, `SortOpt`
- Depends on: `yazi-shared`
- Used by: `yazi-actor` implementations

**Renderer (`Root`):**
- Purpose: Compose the ratatui `Widget` tree; delegate to Lua for the base UI layout; render overlays in z-order
- Location: `yazi-fm/src/root.rs`
- Contains: `Root::render()` calls Lua `Root:new(area):redraw()`, then renders `Preview`, `Modal`, and all visible overlays
- Depends on: `yazi-binding`, `yazi-plugin::LUA`, `yazi-core::Core`
- Used by: `App::render()`

**Lua Plugin System:**
- Purpose: Provide a Lua 5.5 scripting environment; expose Rust state as read-only `UserData` via `Lives` scoped bindings
- Location: `yazi-plugin/src/`, `yazi-binding/src/`, `yazi-actor/src/lives/`
- Contains: `Lives::scope()` pins `Core` as a Lua global `cx`; `yazi-binding` exposes element types (Rect, Style, etc.) and state accessors
- Depends on: `mlua`, `yazi-core`, `yazi-config`
- Used by: `Root::render()`, `App::render()`, `AcceptPayload` actor

**Scheduler:**
- Purpose: Execute file operations, preload/fetch tasks, plugin workers, and external processes with configurable concurrency and priority queues
- Location: `yazi-scheduler/src/`
- Contains: `fetch/`, `file/`, `plugin/`, `preload/`, `process/`, `size/` task types; a `Scheduler` runner
- Depends on: `yazi-proxy`, `yazi-fs`, `yazi-plugin`
- Used by: `yazi-core::Tasks`

**DDS (Data Distribution Service):**
- Purpose: IPC broker — Unix domain socket server that routes typed `Payload` messages between yazi instances and the `ya` CLI
- Location: `yazi-dds/src/`
- Contains: `Server` (accepts connections, routes by receiver ID and `ability`), `Client` (connects, sends/receives), `Pubsub` (local/remote event subscriptions), `Payload`/`Ember` types
- Depends on: `tokio`, `yazi-shared`, `yazi-boot`
- Used by: `main()` (calls `yazi_dds::serve()`), `yazi-cli` for `emit`/`pub`/`sub` commands

**VFS / SFTP:**
- Purpose: Abstract filesystem access; local filesystem ops go through `yazi-fs`; remote (SFTP) access goes through `yazi-sftp` + `yazi-vfs::provider`
- Location: `yazi-vfs/src/`, `yazi-sftp/src/`
- Contains: `VfsFile`, `provider::` with local and SFTP providers; `yazi-sftp` implements the SSH/SFTP protocol via `russh`
- Depends on: `russh`, `yazi-fs`, `tokio`
- Used by: `yazi-actor` cd/download/upload actors, `yazi-core`

**Image Adapter:**
- Purpose: Detect terminal emulator capabilities and render inline images using the best available protocol
- Location: `yazi-adapter/src/`
- Contains: `Adapter` enum (Kgp, KgpOld, Iip, Sixel, Chafa, Ueberzug); `drivers/` with one file per protocol; `yazi-emulator` handles brand/capability detection
- Depends on: `yazi-emulator`, `ratatui`
- Used by: preview rendering in `yazi-fm`

## Data Flow

**Key press to screen update:**

1. `crossterm` event arrives on `Signals` task → `Event::Key` emitted to shared MPSC channel
2. `App::serve()` drains up to 50 events per batch
3. `Dispatcher::dispatch_key()` → `Router::route()` looks up keymap for current `Layer` → emits `Event::Seq` or `Event::Call`
4. `Dispatcher::dispatch_call()` → `Executor::execute()` matches layer + action name → calls `act!(layer:name, cx, action)`
5. `Actor::act()` mutates `Core` fields, optionally spawns async tasks via `tokio::spawn` with result posted back through `Proxy::*` → `emit!(Call(...))`
6. After each event, `NEED_RENDER` atomic is checked; if set and ≥10ms since last frame, `App::render()` is called
7. `Root::render()` calls Lua `cx` (populated via `Lives::scope`), then renders Rust overlay widgets into ratatui `Buffer`
8. `Term::draw()` diffs buffer and writes escape sequences via `crossterm` + `TTY`

**DDS message from `ya emit`:**

1. `ya` CLI connects to Unix socket, sends `hi` then payload then `bye`
2. `yazi-dds::Server` routes to connected yazi instance(s) by receiver ID and ability set
3. Receiving yazi instance's `Client` loop receives line, parses `Payload`, calls `payload.emit()` → `Event::Call(accept_payload)`
4. `AcceptPayload::act()` looks up registered Lua handlers in `LOCAL`/`REMOTE` pubsub tables, calls each via `LUA`

**State management:**

- `Core` is owned by `App` and mutated synchronously on the Tokio local set thread
- Background tasks (async Tokio tasks) never hold `&mut Core`; they post results back via `Proxy` functions that emit `Event::Call`
- Lua scripts access a read-only snapshot of state through `Lives`-scoped `UserData` bound to the current `Core` reference

## Key Abstractions

**`Actor` trait:**
- Purpose: Uniform interface for every command in the system
- Examples: `yazi-actor/src/mgr/cd.rs`, `yazi-actor/src/mgr/open.rs`, `yazi-actor/src/app/bootstrap.rs`
- Pattern: Zero-size structs; `type Options` parsed from `ActionCow`; `fn act(cx, opt) -> Result<Data>`; optional `fn hook()` for post-action DDS sparks

**`Event` enum:**
- Purpose: Central message bus shared between terminal input, action dispatch, and render scheduling
- Location: `yazi-shared/src/event/event.rs`
- Variants: `Call(ActionCow)`, `Seq(Vec<ActionCow>)`, `Render(bool)`, `Key`, `Mouse`, `Resize`, `Focus`, `Paste`

**`Layer` enum:**
- Purpose: Tracks which UI overlay is topmost, determining keymap lookup and executor routing
- Location: `yazi-shared/src/layer.rs`
- Values: `Mgr`, `Tasks`, `Spot`, `Pick`, `Input`, `Confirm`, `Help`, `Cmp`, `Which`, `App`, `Notify`

**`Ctx` struct:**
- Purpose: Short-lived mutable borrow of `Core` enriched with active tab index and action source; passed to every `Actor::act()`
- Location: `yazi-actor/src/context.rs`
- Pattern: Created by `Ctx::new(&action, core, term)` at start of `Executor` method; `Deref` into `Core` for ergonomic access

**`RoCell` / `SyncCell`:**
- Purpose: Thread-safe write-once containers used as module-level globals (replacing `lazy_static` / `OnceLock`)
- Location: `yazi-shared/src/ro_cell.rs`, `yazi-shared/src/sync_cell.rs`
- Pattern: `pub static FOO: RoCell<T> = RoCell::new(); ... FOO.init(value);`

## Entry Points

**`yazi-fm` binary:**
- Location: `yazi-fm/src/main.rs`
- Triggers: User runs `yazi` in terminal
- Responsibilities: Calls `init()` on every crate in dependency order; starts DDS `serve()`; runs `App::serve()` on Tokio local set

**`yazi-cli` binary (`ya`):**
- Location: `yazi-cli/src/main.rs`
- Triggers: User runs `ya emit|pub|sub|pkg`
- Responsibilities: Minimal init (shared + fs only); connects to running yazi DDS server for IPC commands; manages plugin packages

## Error Handling

**Strategy:** `anyhow::Result<Data>` propagated through the actor chain; dispatcher logs errors at `warn` level and continues the loop — failures do not crash the app

**Patterns:**
- `act!` macro returns `Result<Data>`; callers can use `?` to propagate or `.ok()` to silently ignore
- `succ!()` macro returns `Ok(Data::default())` — used to terminate actors that have nothing to return
- `err!(expr)` macro logs expression result as a tracing `error` but does not propagate
- `render!()` macro emits a `Render` event from within `succ!()` when state changed

## Cross-Cutting Concerns

**Logging:** `tracing` crate; level controlled via `YAZI_LOG` env var; initialized in `yazi-fm/src/logs.rs`

**Validation:** Config validation happens at `yazi-config::init()` — parse errors show a blocking prompt then fall back to preset defaults

**Authentication:** N/A for local mode; SFTP uses `russh` with key-based or password auth negotiated in `yazi-sftp/src/session.rs`

**Concurrency model:** Single Tokio `LocalSet` thread owns `Core` and runs the event loop; all background I/O is spawned as regular `tokio::spawn` tasks and communicate back via `Event` channel

---

*Architecture analysis: 2026-03-21*
