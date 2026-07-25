# AGENTS.md

## Rules

- Applies repo-wide. Keep changes scoped; do not create issues or pull requests, or post comments.

## Project

- Rust 2024 Cargo workspace containing all `yazi-*` crates; default members are `yazi-fm` (`yazi`) and `yazi-cli` (`ya`).

## Style

- Follow nearby code and use idiomatic Rust and Lua. Rust uses `snake_case` for modules, functions, and fields and `PascalCase` for types, traits, and variants. Lua uses PascalCase component tables, `local M` plugin modules, `snake_case` methods/locals, and `_name` private fields.
- Preserve established terms and type families: `Url`/`UrlBuf`/`UrlCow`, `PathDyn`/`PathBufDyn`/`PathCow`, `*Ref`, `*Arc`, `*Opt`, `*State`, `*Job`, `*Prog`, `File`, `Folder`, `Tab`, `Mgr`, and `Task`. Use `Url` for logical locations and `Path` for filesystem paths.
- Reuse established plugin and event names (`fetch`, `preload`, `peek`, `seek`, `spot`, `entry`, `setup`, `yank`, `hover`, and `select`) across Rust, Lua, and configuration.
- Use Rust prefixes (`as_`, `to_`, `into_`, `try_`, `is_`, `has_`) according to their usual semantics; prefer descriptive names.
- Name variables, modules, methods, and other symbols simply, elegantly, and expressively. Be creative while keeping names clear, consistent with established terminology, and idiomatic.
- When passing arguments, use the parameter's conversion traits directly (such as `Into<_>` or `AsRef<_>`); avoid eager conversions like `.to_string()`, `.to_owned()`, and `.as_ref()` unless ownership, type inference, or semantics require them.
- Prefer general-purpose traits and conversion APIs already provided by the codebase or its dependencies over manual construction or adapter closures; for example, use `into_lua()` where applicable.

## Code Changes

- Search and reuse first. For new features, extend existing infrastructure or data structures with general, reusable capabilities when that keeps the final code concise.
- For refactors, inspect the whole target module and its callers first. Look for duplicated work, redundant I/O, underpowered return values, one-use wrappers, and reusable cross-platform abstractions; implement high-confidence, behavior-preserving simplifications while preserving error, fallback, and platform semantics.
- Keep diffs minimal and avoid unrelated refactors. Prefer clear code over custom patterns or comments; comment only behavior the code cannot explain.
- Put reusable code in the lowest suitable shared layer; avoid unnecessary dependencies and allocations. Prefer borrowed values and existing wrappers.
- Use stable Rust APIs; nightly is formatting-only. Use only `pub`, `pub(super)`, and `pub(crate)`—never `pub(in ...)`.
- Keep async I/O non-blocking, preserve platform/fork behavior, and follow existing error boundaries with `?`.
- For renames or refactors, update all related variables, functions, parameters, modules, methods, types, derived types, exports, tests, configuration keys, documentation, and Lua bindings.
- Do not add or modify tests unless requested.

## Validation

- Prefer targeted debug checks; use multiple `-p` flags for affected crates before the whole workspace.

```sh
cargo check -p <package>
cargo test -p <package>
cargo clippy -p <package>
find . -name '*.rs' -not -path './target/*' -exec rustfmt +nightly --check {} +
stylua --color always --check .
```

- Use `cargo check` instead of `cargo build` unless artifacts are needed. Do not use `--release` unless requested; use `scripts/build.sh <target>` for release or cross-target packaging.
- Run relevant existing tests when needed, then inspect `git diff` and verify that only intended files changed.
