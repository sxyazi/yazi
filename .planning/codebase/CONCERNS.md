# Codebase Concerns

**Analysis Date:** 2026-03-21

---

## Tech Debt

**SFTP URL rebase not implemented:**
- Issue: `UrlBuf::rebase()` panics at runtime for SFTP URLs via `todo!()` macro — hardest-blocking panic
- Files: `yazi-shared/src/url/buf.rs:166`
- Impact: Any code path that calls `rebase()` on an SFTP URL will unconditionally panic at runtime
- Fix approach: Implement the commented-out `Self::Sftp { loc: loc.rebase(base), domain: domain.clone() }` arm

**`RwFile::metadata()` uses placeholder string for path:**
- Issue: Both Tokio and SFTP arms pass the literal `"// FIXME"` string as the file path
- Files: `yazi-vfs/src/provider/rw_file.rs:23-24`
- Impact: Any metadata query on an open file returns incorrect path information; affects file attributes display
- Fix approach: Capture and store the path when the file is opened; pass it through to `Cha::new()`

**Splatter legacy `%*` shorthand pending removal:**
- Issue: Three `// TODO: remove` comments mark the `%*` variant (maps to `visit_selected`) as deprecated; dead code kept for backward compat
- Files: `yazi-fs/src/splatter.rs:85,158,247,346`
- Impact: Bloated match arms, confusing maintenance; breaking change deferred
- Fix approach: Remove `%*` handling after a deprecation grace period; update user-facing docs

**`open_shell_compat` compatibility shim:**
- Issue: Entire `OpenShellCompat` actor, `TasksProxy::open_shell_compat()`, and `match_and_open()` helper all carry `// TODO: remove` comments
- Files: `yazi-actor/src/tasks/open_shell_compat.rs`, `yazi-proxy/src/tasks.rs:10`, `yazi-actor/src/mgr/open_do.rs:61`
- Impact: Multiple call sites continue routing through legacy path instead of new `process_open`
- Fix approach: Migrate all callers to use `TasksProxy::process_exec()` directly; delete the shim actor

**`ProcessOpenOpt.spread` field marked for removal:**
- Issue: `spread: bool` field carries `// TODO: remove` annotation in the parser type
- Files: `yazi-parser/src/tasks/process_open.rs:17`
- Impact: Dead struct field increasing complexity of every `ProcessOpenOpt` construction
- Fix approach: Remove field; propagate removal to all construction sites in `mgr/shell.rs`, `mgr/open_do.rs`, `tasks/process_open.rs`

**`ChaMode::is_exec()` deprecated but still in active use:**
- Issue: Method carries `// TODO: deprecate` comment yet is called from `yazi-binding`, `yazi-config` theme code
- Files: `yazi-fs/src/cha/mode.rs:153`, `yazi-binding/src/cha.rs:64`, `yazi-config/src/theme/is.rs:33`, `yazi-config/src/theme/icon.rs:66`
- Impact: Deprecated API surface remains public and actively maintained
- Fix approach: Determine replacement API, update callers, then remove method

**`vfs::unique_name()` deprecated but infrastructure retained:**
- Issue: `yazi_vfs::unique_name()` is deprecated in favour of `fs.unique()`; Lua wrapper still exposes `fs.unique_name()` with a deprecation warning at runtime
- Files: `yazi-vfs/src/fns.rs:16`, `yazi-plugin/src/fs/fs.rs:234-241`
- Impact: Duplicate code path; external plugins still using the old API receive runtime warnings
- Fix approach: Track down plugin usages via community; remove API after sufficient migration time

**`Data::into_any2()` has no satisfactory name:**
- Issue: Method acknowledged as needing a better name in comment `// FIXME: find a better name`
- Files: `yazi-shared/src/data/data.rs:233`
- Impact: Poor discoverability in the `Data` API surface
- Fix approach: Audit all callsites and rename; update callers

**Multiple FIXME-annotated compat `impl AsRef` / `impl Default` on URL types:**
- Issue: `LocBuf`, `UrlCow`, `UrlBuf`, `LocBuf` all carry `// FIXME: remove` impls that exist only for transitional compat
- Files: `yazi-shared/src/loc/buf.rs:23`, `yazi-shared/src/url/cow.rs:21`, `yazi-shared/src/url/buf.rs:16`, `yazi-shared/src/loc/loc.rs:39`
- Impact: These impls can mask type errors; they block further API hardening
- Fix approach: Identify callers relying on these impls; replace with explicit conversions; remove impls

**Lua `ya.select()` stub not implemented:**
- Issue: `ya.select()` is registered but returns `Ok(())` unconditionally; body is a TODO placeholder
- Files: `yazi-plugin/src/utils/sync.rs:126-128`
- Impact: Any plugin that attempts to call `ya.select()` silently succeeds with no result; misleading API
- Fix approach: Implement select semantics or return an explicit "not implemented" Lua error until complete

**`spot:copy "line"` unimplemented:**
- Issue: The `"line"` copy type in the spot view has an empty TODO body; no text is copied
- Files: `yazi-actor/src/spot/copy.rs:28`
- Impact: Silent no-op when user copies a line from spot view
- Fix approach: Implement line extraction from the table widget

**`ya.layer()` cursor and list fields unset:**
- Issue: `cursor: None` and `list: Default::default()` in `ya.input()` / `ya.confirm()` carry `// TODO` comments indicating incomplete options plumbing
- Files: `yazi-plugin/src/utils/layer.rs:57,89`
- Impact: Plugins cannot programmatically position the cursor or pre-populate confirm lists via Lua
- Fix approach: Thread these fields through `InputCfg` / `ConfirmCfg` from the Lua table argument

**`WATCHER` semaphore acquire unwrap in file watcher:**
- Issue: `WATCHER.acquire().await.unwrap()` can panic if the semaphore is closed during shutdown
- Files: `yazi-watcher/src/local/local.rs:96`
- Impact: Panics possible during clean shutdown sequence
- Fix approach: Use `?` propagation or check for `SemaphoreAcquireError` explicitly

---

## Known Bugs

**DDS `pub_inner_hi()` error silently swallowed:**
- Symptoms: Peer hello messages fail without any error propagated or logged
- Files: `yazi-dds/src/pubsub.rs:124-125`
- Trigger: IPC transport error during initial hi broadcast
- Workaround: None; failure is invisible

**Watcher notification batching workaround:**
- Symptoms: File change events are coalesced over 250 ms windows with batch size 1000 as an acknowledged workaround
- Files: `yazi-watcher/src/local/local.rs:89`
- Trigger: Comment says `// TODO: revert this once a new notification is implemented`; current impl is not the intended design
- Workaround: Current behaviour is functional but has fixed latency overhead

**LINKED returns Path type instead of Url:**
- Symptoms: Symlink-based watch targets use an incorrect type, requiring an extra `Url::regular()` wrapping
- Files: `yazi-watcher/src/reporter.rs:33-35`
- Trigger: Any access to watched symlinked directories
- Workaround: Current adapter in place but acknowledged as wrong

**`UrlCov::parent()` not used in selection remove path:**
- Symptoms: Parent traversal in `selected.rs` uses a manual workaround instead of the correct `UrlCov::parent()` API
- Files: `yazi-core/src/tab/selected.rs:111`
- Trigger: Removing selected files that have deep parent chains
- Workaround: Manual `while let Some(u) = parent` loop

---

## Security Considerations

**`unsafe std::env::set_var()` in async context:**
- Risk: `set_var` is unsound in multi-threaded programs; marked `unsafe` in Rust 1.83+. Called from a `spawn_blocking` closure that runs concurrently with other threads reading the environment
- Files: `yazi-fs/src/cwd.rs:101,108`, `yazi-dds/src/lib.rs:25-33`
- Current mitigation: `unsafe` block is present acknowledging the risk; init happens early in `dds::init()`
- Recommendations: Migrate environment variables to explicit `Arc<OsString>` passing or use `std::env::set_var` only before any threads are spawned

**`str::from_utf8_unchecked()` without SAFETY comments:**
- Risk: If upstream data is not validated as valid UTF-8 before calling the unchecked variant, memory safety of `str` references is violated
- Files: `yazi-adapter/src/drivers/kgp.rs:360,366`, `yazi-adapter/src/drivers/kgp_old.rs:49,55`, `yazi-plugin/src/external/highlighter.rs:188`, `yazi-shared/src/strand/buf.rs:68,85,90`
- Current mitigation: No `// SAFETY` comment explaining the invariant; workspace lint `missing_safety_doc = "allow"` suppresses the Clippy warning globally (`Cargo.toml:87`)
- Recommendations: Add `// SAFETY:` comments to each site explaining why the bytes are valid UTF-8; re-enable `missing_safety_doc` lint

**`missing_safety_doc` lint globally suppressed:**
- Risk: All `unsafe fn` blocks in the workspace are exempt from documentation requirements
- Files: `Cargo.toml:87`
- Current mitigation: None
- Recommendations: Remove `missing_safety_doc = "allow"` from workspace lints; fix each site

---

## Performance Bottlenecks

**`Refresh::trigger_dirs()` acknowledged slow path:**
- Problem: Spawns async tasks to stat and reload every visible folder on every refresh event; acknowledged in comment `// TODO: performance improvement`
- Files: `yazi-actor/src/mgr/refresh.rs:44`
- Cause: No caching layer; unconditional `Files::assert_stale()` and `Files::from_dir_bulk()` calls per folder
- Improvement path: Track change timestamps and skip re-reads when stat hasn't changed; debounce refresh triggers

**`icon()` method computes icon match on every call without caching:**
- Problem: `yazi-actor/src/lives/file.rs:91` and `yazi-binding/src/file.rs:93` both call `THEME.icon.matches()` per file on every render with `// TODO: use a cache`
- Files: `yazi-actor/src/lives/file.rs:91`, `yazi-binding/src/file.rs:93`
- Cause: No memoization on `File` for the computed icon
- Improvement path: Cache the icon result on the `File` struct after first computation; invalidate on theme reload

**`Pos::to_rect()` caching absent:**
- Problem: Rect computation from `Pos` has `// TODO: cache` comment indicating repeated recomputation
- Files: `yazi-binding/src/elements/pos.rs:84`
- Cause: No cached field
- Improvement path: Store computed `Rect` alongside `Pos`; invalidate when terminal size changes

**245 `.clone()` calls across the codebase:**
- Problem: URL types (`UrlBuf`, `UrlCow`) are cloned pervasively; 126 `unwrap()`/`expect()` calls in non-test production code
- Files: spread across all crates
- Cause: URL types carry owned strings with no interning beyond the `Pool` for symbols; clone-heavy IPC message passing
- Improvement path: Audit hot paths using `clone()` on URLs; leverage existing `UrlCow` borrow types where ownership is not needed

---

## Fragile Areas

**`yazi-fs/src/splatter.rs` — shell argument expansion:**
- Files: `yazi-fs/src/splatter.rs`
- Why fragile: Multiple `unreachable!()` arms in `visit_hovered()`, `visit_dirname()` etc. that panic if the character dispatch logic in `visit()` is ever inconsistent; legacy `%*` arms awaiting removal add more match paths
- Safe modification: Run full `cargo test -p yazi-fs` after any change; ensure all `visit_*` dispatch matches the `visit()` character table exactly
- Test coverage: Inline `#[test]` block exists for basic cases; no exhaustive round-trip tests for all interpolation forms

**`yazi-shared/src/url/` — URL type hierarchy:**
- Files: `yazi-shared/src/url/url.rs`, `yazi-shared/src/url/buf.rs`, `yazi-shared/src/url/cow.rs`
- Why fragile: Parallel enum hierarchies (`Url`, `UrlBuf`, `UrlCow`) must stay in sync; SFTP arm incomplete in `rebase()`; several compat `impl` blocks marked for removal; any variant addition requires changes in all three types plus `Loc`/`LocBuf`
- Safe modification: Add variants to all three enums simultaneously; verify no `_ =>` arms silently absorb new variants; check all `From`/`Into` impls
- Test coverage: Unit tests exist in `yazi-shared/src/url/buf.rs` and `cow.rs` but do not cover SFTP variant paths

**`yazi-dds` — inter-process pub/sub:**
- Files: `yazi-dds/src/pubsub.rs`, `yazi-dds/src/lib.rs`
- Why fragile: Uses `unsafe { std::env::set_var }` during init; `pub_inner_hi()` silently ignores errors; global statics (`ID`, `PEERS`, `LOCAL`, `REMOTE`) initialized via `RoCell::init()` which panics on double-init
- Safe modification: Only call `dds::init()` once before spawning any async tasks; never call from tests without isolation
- Test coverage: No tests found in `yazi-dds/`

**`yazi-adapter` — image protocol drivers:**
- Files: `yazi-adapter/src/drivers/kgp.rs`, `yazi-adapter/src/drivers/kgp_old.rs`
- Why fragile: Two parallel KGP driver implementations exist (current and `_old`); both use `str::from_utf8_unchecked()` without SAFETY docs; base64-encoded image data passed through format strings
- Safe modification: Treat `kgp_old.rs` as read-only reference; do not add features there; changes to base64 chunking must preserve chunk boundary alignment
- Test coverage: No unit tests found; runtime-only validation

**`yazi-plugin/src/isolate/` — Lua plugin isolation:**
- Files: `yazi-plugin/src/isolate/peek.rs`, `yazi-plugin/src/isolate/preload.rs`, `yazi-plugin/src/isolate/fetch.rs`, `yazi-plugin/src/isolate/spot.rs`
- Why fragile: Uses `Handle::current().block_on()` which blocks the calling OS thread; if called from within an async context on the Tokio thread pool this starves the executor
- Safe modification: Only invoke isolate entry points from dedicated `spawn_blocking` threads, never from `async` tasks directly
- Test coverage: No unit tests; integration-tested only via running yazi

---

## Scaling Limits

**Lua `ya.select()` stub:**
- Current capacity: Not functional; always returns empty result
- Limit: Any plugin relying on multi-item selection UI is broken
- Scaling path: Implement in `yazi-plugin/src/utils/sync.rs`

**File watcher batch size hardcoded at 1000:**
- Current capacity: 1000 change events per 250 ms window
- Limit: Repositories or directories with high churn (e.g., build output) may overwhelm the batch and drop events
- Scaling path: Make `chunks_timeout` size/duration configurable; implement proper OS-level coalescing

---

## Dependencies at Risk

**`ratatui` unstable features:**
- Risk: Uses `unstable-rendered-line-info` and `unstable-widget-ref` feature flags; these APIs can break without a semver guarantee between minor ratatui versions
- Impact: Build breaks or behavioural regressions on ratatui upgrades
- Migration plan: Track ratatui changelog; stabilisation of these features is planned upstream but no ETA

**`mlua` with Lua 5.5:**
- Risk: `lua55` feature targets Lua 5.5 which is not yet a stable release; API changes between Lua 5.5 preview builds could require code updates
- Impact: `yazi-plugin` and `yazi-binding` crates rely on Lua 5.5 semantics
- Migration plan: Pin to a tested mlua version; test against new Lua 5.5 release candidates in CI

---

## Test Coverage Gaps

**`yazi-dds` — zero tests:**
- What's not tested: IPC message serialization, pub/sub routing, peer discovery, all `pubsub.rs` paths
- Files: `yazi-dds/src/pubsub.rs`, `yazi-dds/src/client.rs`, `yazi-dds/src/server.rs`
- Risk: Silent regressions in cross-instance communication; `pub_inner_hi()` error swallowing undetected
- Priority: High

**`yazi-adapter` — zero tests:**
- What's not tested: All image protocol drivers (KGP, KGP-old, Sixel, IIP, Ueberzug), ICC colour transform, image encoding
- Files: `yazi-adapter/src/drivers/`, `yazi-adapter/src/icc.rs`
- Risk: Image rendering regressions not caught until runtime
- Priority: High

**`yazi-vfs` SFTP provider paths — sparse tests:**
- What's not tested: `RwFile` metadata with SFTP, `try_absolute()` for archive URLs, SFTP-specific `Cha` conversion
- Files: `yazi-vfs/src/provider/rw_file.rs`, `yazi-vfs/src/provider/provider.rs`
- Risk: Archive and SFTP URL handling silently broken; placeholder `"// FIXME"` path in production unnoticed
- Priority: High

**`yazi-actor` core actions — partial tests:**
- What's not tested: `bulk_rename` cycle detection edge cases, `open_do::match_and_open()` legacy path, `spot::copy "line"` variant
- Files: `yazi-actor/src/mgr/bulk_rename.rs`, `yazi-actor/src/mgr/open_do.rs`, `yazi-actor/src/spot/copy.rs`
- Risk: Silent regressions in file operations; no-op copy undetected
- Priority: Medium

**Only 23 of 912 Rust source files contain tests (~2.5%):**
- What's not tested: The vast majority of crate logic including scheduler, config parsing, most of the actor layer, watcher, and binding
- Files: All crates except `yazi-shared`, `yazi-fs`, `yazi-core` (partial), `yazi-config` (partial)
- Risk: Wide surface area for undetected regressions; TDD mandate in CLAUDE.md is not yet reflected in coverage
- Priority: High

---

*Concerns audit: 2026-03-21*
