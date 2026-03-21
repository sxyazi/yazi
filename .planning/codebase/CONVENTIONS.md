# Coding Conventions

**Analysis Date:** 2026-03-21

## Naming Patterns

**Files:**
- `snake_case.rs` for all Rust source files
- Module directories named after the concept they contain (e.g., `yazi-shared/src/url/`, `yazi-fs/src/path/`)
- Error types live in dedicated `error.rs` or `error/` files per crate
- Config presets in `preset/` subdirectories

**Structs and Enums:**
- `PascalCase` for all types: `UrlBuf`, `LocBuf`, `Backstack`, `Selected`, `SchemeCow`
- `Buf`-suffixed types are owned buffers; reference types use the bare name (e.g., `Url` vs `UrlBuf`, `Loc` vs `LocBuf`)
- `Cow`-suffixed types are clone-on-write wrappers (e.g., `UrlCow`, `SchemeCow`, `PathCow`)
- Error enums use `Error` suffix with domain prefix: `PathDynError`, `StripPrefixError`, `SetNameError`

**Functions:**
- `snake_case` for all functions and methods
- Boolean getters often use `is_` prefix: `is_empty()`, `is_dir()`, `is_local()`, `is_regular()`
- Public accessor stubs that return a borrowed view often match the field name: `uri()`, `urn()`, `base()`, `trail()`
- Fallible constructors use `try_` prefix: `try_join()`, `try_set_name()`, `try_strip_prefix()`
- Conversion constructors named by intent: `zeroed()`, `floated()`, `saturated()`, `from_components()`
- Async variants of sync functions stay named the same (not `_async` suffixed)

**Variables:**
- `snake_case`, single-letter names only for iterators (`i`, `c`, `b`) and closures
- Avoid abbreviations; prefer `url`, `path`, `domain` over `u`, `p`, `d` in function signatures
- Tuple/temporary intermediate values named `a`, `b` when context is obvious

**Traits:**
- `PascalCase`; `Able` suffix for marker/capability traits: `LocAble`, `LocBufAble`, `Splatable`
- `Like` suffix for view-oriented traits: `UrlLike`, `PathLike`, `SchemeLike`, `StrandLike`
- `As` prefix for conversion traits: `AsUrl`, `AsPath`, `AsScheme`, `AsStrand`

**Constants and Statics:**
- `UPPER_SNAKE_CASE` for statics: `ADAPTOR`, `WSL`, `WATCHED`, `WATCHER`
- Associated constants inside `impl` use `UPPER_SNAKE_CASE`: `const NAME: &str = "bulk_rename"`

## Code Style

**Formatting:**
- Tool: `rustfmt` (nightly toolchain required)
- Config: `rustfmt.toml` at repo root
- Hard tabs for indentation (not spaces)
- Tab width: 2 spaces equivalent
- Edition 2024 style
- Imports grouped: std → external crates → workspace crates (via `group_imports = "StdExternalCrate"`)
- Imports within a group horizontally collapsed: `use std::{borrow::Cow, ffi::OsStr, ...}`
- One-liner functions on a single line when possible (`fn_single_line = true`)
- Struct field alignment enabled up to 99 chars: fields visually aligned with spaces

**Linting:**
- Tool: Clippy (stable)
- Config: `[workspace.lints.clippy]` in `Cargo.toml`
- `format_push_string = "warn"` – avoid `push_str(&format!(...))`, use `write!` instead
- `implicit_clone = "warn"` – `.to_owned()` or `.clone()` must be explicit
- `use_self = "warn"` – use `Self` instead of the concrete type name inside `impl` blocks
- `if_same_then_else = "allow"` – duplicate branches permitted
- `module_inception = "allow"` – `mod foo { struct Foo }` pattern is allowed

**Lua:**
- Tool: StyLua
- Config: `stylua.toml` at repo root
- Lua 5.4 syntax
- Indent width: 2 spaces
- `call_parentheses = "NoSingleTable"` – omit parentheses when the only argument is a table
- `sort_requires = true` – `require()` calls are sorted alphabetically

## Import Organization

**Rust order (enforced by rustfmt):**
1. `std` / `core` / `alloc`
2. External crates (e.g., `anyhow`, `mlua`, `tokio`)
3. Workspace crates (e.g., `yazi_shared`, `yazi_fs`)
4. Local / crate-internal: `crate::`, `super::`, `self::`

**Path Aliases:**
- No `use` aliases used in production code; type aliases via `type` declaration where needed
- In tests, `use super::*` is the universal import pattern

**Module Export Pattern:**
- `yazi_macro::mod_pub!(a b c)` – re-exports submodules as `pub mod` (subdirectories visible)
- `yazi_macro::mod_flat!(a b c)` – makes private `mod` items `pub use`d flat into current namespace
- Avoids manual `mod x; pub use x::*;` repetition throughout all crates

## Error Handling

**Boundary strategy:**
- Functions that can fail return `anyhow::Result<T>` at application layer (actors, async tasks)
- Domain-specific errors use `thiserror::Error`-derived enums with precise variants (see `yazi-shared/src/path/error.rs`, `yazi-shared/src/strand/error.rs`)
- Error propagation uses `?` operator; `.context()` added where clarification helps
- `bail!()` from `anyhow` used for early failure with message
- `ensure!()` from `anyhow` used for condition checks

**Error macro (`err!`):**
- `yazi_macro::err!(expr)` – logs error via `tracing::error!` and discards result; used for fire-and-forget side effects
- `yazi_macro::err!(expr, "fmt {}", args)` – same but with custom message
- Do NOT use `unwrap()` in production code; `expect("message")` is acceptable only when invariant is guaranteed
- `debug_assert!(...)` used inside `impl` blocks to verify post-conditions cheaply

**Panics:**
- `unreachable!()` used only inside exhaustive match arms that should logically never be hit
- `expect("...")` used in constructors/init code where failure means programmer error

## Logging

**Framework:** `tracing` crate

**Import pattern:**
```rust
use tracing::{debug, error, warn};
// or targeted:
use tracing::error;
```

**Patterns:**
- `tracing::error!(...)` – non-recoverable errors in async tasks
- `tracing::warn!(...)` – degraded state that can continue
- `tracing::debug!(...)` – diagnostic information (only in debug builds via `max_level_debug`)
- `yazi_macro::err!(expr)` – shorthand to log and discard a `Result::Err`
- `yazi_macro::time!(expr)` – wraps an expression in a debug-level timing log
- Log level controlled at runtime via `YAZI_LOG` environment variable

## Comments

**When to Comment:**
- Inline comments explain WHY, not WHAT: `// Only keep 30 URLs before the cursor, the cleanup threshold is 60`
- Code sections within large files use `// ---` separator lines: `// --- Tuple`, `// --- Tests`
- Platform-conditional code annotated: `// "/" is both a directory separator and the root directory per se`
- Pending work uses `// TODO:` or `// FIXME:` markers (not `//TODO:`)

**Doc comments (`///`):**
- Used only on public API items with non-obvious contracts
- CLI arguments all have `///` docs (powers `--help` output via `clap`)
- Internal implementation functions rarely have doc comments

**Commented-out code:**
- Not present; dead code is removed immediately

## Function Design

**Size:** Typically 5–25 lines; business logic delegated to private helpers

**Parameters:** Max 3 in most cases; larger argument sets use an Options struct (e.g., `BulkRenameOpt`, `ShowOpt`)

**Return Values:**
- `Option<T>` for absence; `Result<T>` for failure
- `bool` return for mutation success/failure: `add()`, `remove()` on `Selected`
- Avoid `(bool, T)` tuples; use named return types or structs

**Boolean Parameters:**
- Avoided in public APIs; use enums or named Options structs instead
- Acceptable in private helpers when the call site is immediately adjacent

## Module Design

**Exports:**
- All crates have a `pub fn init()` entry point for global state initialization
- Each crate exposes a flat public API via `mod_flat!` – callers do not need to know internal module layout
- Sub-namespaces exposed via `mod_pub!` when logical grouping is useful (e.g., `elements/`, `path/`)

**Barrel Files:**
- `lib.rs` serves as the crate root; uses macros instead of manual barrel re-exports
- No separate `mod.rs` barrel files – directory modules are declared in `lib.rs`

**Struct Internals:**
- Private fields by default; getters expose them where needed
- `pub(super)` used for intra-module access (e.g., `pub(super) inner`, `pub(super) area`)
- Newtype pattern common for domain types: `struct Backstack { cursor: usize, stack: Vec<UrlBuf> }`

---

*Convention analysis: 2026-03-21*
