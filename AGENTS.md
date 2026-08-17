# AGENTS.md

## Rules

- Applies repo-wide. Keep changes scoped; do not create issues or pull requests, or post comments.

## Project

- Rust 2024 Cargo workspace containing all `yazi-*` crates; default members are `yazi-fm` (`yazi`) and `yazi-cli` (`ya`).

## Style

- Follow nearby code and use idiomatic Rust and Lua. Rust uses `snake_case` for modules, functions, and fields and `PascalCase` for types, traits, and variants. Lua uses PascalCase component tables, `local M` plugin modules, `snake_case` methods/locals, and `_name` private fields.
- Preserve established terms and type families: `Url`/`UrlBuf`/`UrlCow`, `PathDyn`/`PathBufDyn`/`PathCow`, `*Ref`, `*Arc`, `*Opt`, `*State`, `*Job`, `*Prog`, `File`, `Folder`, `Tab`, `Mgr`, and `Task`. Use `Url` for logical locations and `Path` for filesystem paths.
- `key()` identifies a file-list entry; `urn()` is the raw URL path tail. Use `key()` for list state and `urn()` for filesystem-path semantics; do not substitute them mechanically.
- Reuse established plugin and event names (`fetch`, `preload`, `peek`, `seek`, `spot`, `entry`, `setup`, `yank`, `hover`, and `select`) across Rust, Lua, and configuration.
- Use Rust prefixes (`as_`, `to_`, `into_`, `try_`, `is_`, `has_`) according to their usual semantics; prefer descriptive names.
- Name variables, modules, methods, and other symbols simply, elegantly, and expressively. Be creative while keeping names clear, consistent with established terminology, and idiomatic.
- When passing arguments, use the parameter's conversion traits directly (such as `Into<_>` or `AsRef<_>`); avoid eager conversions like `.to_string()`, `.to_owned()`, and `.as_ref()` unless ownership, type inference, or semantics require them. At Lua boundaries, prefer `LuaString` to `String`, and use `BorrowedBytes` when string semantics are unnecessary.
- Let Rust infer types whenever the context is sufficient; when an annotation is needed, put it on the binding (`let value: Type = ...`) instead of turbofishing the expression.
- Prefer methods provided by `UrlLike`, `PathLike`, or `StrandLike` directly on the original value (for example, `buf.parent()` over `buf.as_url().parent()`), rather than converting it first with `as_url()`, `dyn_path()`, or `to_strand()`.
- Prefer `&*value` for dereferencing over `AsRef` when both are suitable.
- Prefer general-purpose traits and conversion APIs already provided by the codebase or its dependencies over manual construction or adapter closures; for example, use `into_lua()` where applicable.

## Code Changes

- Search and reuse first. For new features, extend existing infrastructure or data structures with general, reusable capabilities when that keeps the final code concise.
- Use `gh` to read GitHub issues, pull requests, and their discussions.
- For refactors, inspect the whole target module, its callers, and the surrounding lifecycle first. Understand the system's established assumptions before adding local safeguards; distinguish required invariants from acceptable compromises, and ask when that boundary materially affects the design. Look for duplicated work, redundant I/O, underpowered return values, one-use wrappers, and reusable cross-platform abstractions; implement high-confidence, behavior-preserving simplifications while preserving error, fallback, and platform semantics.
- Keep diffs minimal and avoid unrelated refactors, speculative abstractions, and defensive handling for states the system already excludes. Prefer clear, flat control flow, expressions, and positive predicates; use standard combinators, early returns, ordered branches, and match guards to avoid nested conditionals, compound negation, and unnecessary wrapper syntax. When idiomatic and equivalent, prefer visually parallel forms such as `true as usize` over `usize::from(true)`. Comment only behavior the code cannot explain.
- Keep responsibility boundaries clear and cohesive. Prefer pure functions, explicit invariants, and idempotent operations when repeated calls are natural and idempotency removes coordination or state. Favor convention over configuration when invariants can eliminate state or coordination. Put reusable code in the lowest suitable shared layer; avoid unnecessary dependencies and allocations. Prefer borrowed values and existing wrappers.
- Initialize crates explicitly from the application entrypoint in dependency order; a module must not initialize another module as a side effect.
- Use stable Rust APIs; nightly is formatting-only—apply Rust formatting directly with `rustfmt +nightly **/*.rs`. Use only `pub`, `pub(super)`, and `pub(crate)`—never `pub(in ...)`.
- Keep async I/O non-blocking, preserve platform/fork behavior, and follow existing error boundaries with `?`.
- For renames or refactors, update all related variables, functions, parameters, modules, methods, types, derived types, exports, tests, configuration keys, documentation, Lua bindings, and, when a type and file share a name, the file as well; do not preserve renamed terms through aliases or re-exports.
- When adding a changelog entry, leave the PR number blank for a human to fill in.
- Do not add tests or change test behavior unless requested.

## Validation

- Prefer targeted debug checks; use multiple `-p` flags for affected crates before the whole workspace.
- When investigating bugs, add temporary diagnostics when useful (`tracing` in Rust and `ya.dbg` in Lua), reproduce in a simulated terminal with `YAZI_LOG=debug`, and inspect the log file to pinpoint the cause; remove diagnostics before handoff.

```sh
cargo check -p <package>
cargo test -p <package>
cargo clippy -p <package>
rustfmt +nightly **/*.rs
stylua --color always --check .
```

- Use `cargo check` instead of `cargo build` unless artifacts are needed. Do not use `--release` unless requested; use `scripts/build.sh <target>` for release or cross-target packaging.
- Run relevant existing tests when needed, then inspect `git diff` and verify that only intended files changed.
