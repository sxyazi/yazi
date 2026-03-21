# Testing Patterns

**Analysis Date:** 2026-03-21

## Test Framework

**Runner:**
- Rust's built-in `cargo test`
- No external test runner; no `jest.config` or `vitest.config`

**Assertion Library:**
- `assert_eq!`, `assert_ne!`, `assert!` from Rust standard library
- `anyhow::Result` used as test return type for `?` propagation

**Run Commands:**
```bash
cargo test --workspace --verbose    # Run all tests across all crates
cargo test -p yazi-core             # Test a specific crate
cargo test test_backstack           # Run a single test by name
cargo test --workspace              # Full suite (CI equivalent)
```

## Test File Organization

**Location:**
- Co-located: tests live in a `#[cfg(test)] mod tests { ... }` block at the bottom of the source file they test
- No separate `tests/` integration test directories found
- No external test fixtures directory

**Naming:**
- Test module: always named `mod tests`
- Test functions: `test_` prefix followed by the function or concept being tested
  - `fn test_backstack()`, `fn test_natsort()`, `fn test_clean_url()`, `fn test_split()`
  - Behavior-specific names also used without `test_` prefix: `fn insert_many_success()`, `fn test_insert_conflicting_parent()`

**Structure:**
```
src/
├── some_module.rs          # Implementation + inline test module
│   └── #[cfg(test)] mod tests { ... }
├── path/
│   ├── clean.rs            # Tests at bottom of file
│   └── relative.rs         # Tests at bottom of file
```

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;                     // Always import everything from parent

    #[test]
    fn test_something() {
        // arrange
        let mut subject = Subject::default();
        // act + assert
        assert_eq!(subject.method(), expected);
    }

    #[test]
    fn test_something_else() -> anyhow::Result<()> {
        // Use Result return type for tests needing ? propagation
        let u: UrlBuf = "/some/path".parse()?;
        assert_eq!(format!("{u:?}"), "/some/path");
        Ok(())
    }
}
```

**Section Separator:**
```rust
// --- Tests
#[cfg(test)]
mod tests {
```
The `// --- Tests` separator comment appears above the test module in some files.

**Patterns:**
- Setup: `Subject::default()` for zero-state; `yazi_shared::init_tests()` for global state
- No `before_each` / `after_each`; each test is fully self-contained
- Data tables (slices of tuples) used for parameterized assertions
- Helper functions within the `mod tests` block used to reduce assertion boilerplate

## Mocking

**Framework:** None – no mocking crate used

**Patterns:**
- No mock objects; tests call real implementations directly
- Platform-specific tests gated with `#[cfg(unix)]` / `#[cfg(windows)]` on individual test functions
- Global state initialized with `yazi_shared::init_tests()` where needed (see below)
- Tests that require filesystem state initialize `yazi_fs::init()` directly

**What to Mock:**
- Nothing – the codebase uses no mocking framework

**What NOT to Mock:**
- Filesystem, URLs, sorting – all tested with real values

## Fixtures and Factories

**Test Data:**
The dominant pattern is inline slice literals of tuples serving as parameterized test cases:

```rust
let cases = [
    // Comment describing the category
    ("/a/b", "/a/b/c", "c"),
    ("/a/b/c", "/a/b", ".."),
    ("/a/b/d", "/a/b/c", "../c"),
];
for (from, to, expected) in cases {
    let result = function_under_test(from, to);
    assert_eq!(result, expected);
}
```

**Helper closures and functions inside test modules:**
```rust
fn cmp(left: &[&str]) {
    let mut right = left.to_vec();
    right.sort_by(|a, b| natsort(a.as_bytes(), b.as_bytes(), true));
    assert_eq!(left, right);
}

fn matches(glob: &str, url: &str) -> bool {
    Pattern::from_str(glob).unwrap().match_url(UrlCow::try_from(url).unwrap(), false)
}
```

**Global state initialization:**
```rust
#[test]
fn test_something() {
    yazi_shared::init_tests();    // Must call before any URL/path parsing
    yazi_fs::init();              // Must call before CWD-dependent tests
    // ...
}
```
`init_tests()` is idempotent via `OnceLock` so ordering between tests doesn't matter.

**Location:**
- No separate fixtures directory; all test data is inlined in the test block

## Coverage

**Requirements:** Not enforced – no minimum coverage threshold configured

**View Coverage:**
```bash
# No coverage tooling configured in the repository.
# Use cargo-llvm-cov or cargo-tarpaulin manually if needed:
cargo llvm-cov --workspace
```

## Test Types

**Unit Tests:**
- Dominant form: all tests are unit tests co-located with the code they test
- Scope: individual functions and struct methods
- Approach: pure input/output; no I/O or side effects except where `init_tests()` sets up global state

**Integration Tests:**
- Not present as a separate category; complex behavior tested via unit tests against real types

**E2E Tests:**
- Not present

**Property-based Tests:**
- Not present (no `proptest` or `quickcheck` dependencies)

## Common Patterns

**Table-driven tests (most common):**
```rust
let cases: &[(&str, &str)] = &[
    ("input_a", "expected_a"),
    ("input_b", "expected_b"),
];
for &(input, expected) in cases {
    assert_eq!(function(input), expected, "input: {:?}", input);
}
```

**Fallible tests with `anyhow::Result`:**
```rust
#[test]
fn test_join() -> anyhow::Result<()> {
    crate::init_tests();
    let base: UrlBuf = "/a".parse()?;
    assert_eq!(format!("{:?}", base.try_join("b/c")?), "/a/b/c");
    Ok(())
}
```

**Platform-conditional tests:**
```rust
#[cfg(unix)]
#[test]
fn test_split() {
    yazi_shared::init_tests();
    yazi_fs::init();
    // unix-specific assertions
}

#[cfg(windows)]
#[test]
fn test_split() {
    yazi_fs::init();
    // windows-specific assertions
}
```

**Struct state verification tests:**
```rust
#[test]
fn test_remove() {
    let mut s = Selected::default();
    assert!(s.add(Path::new("/a/b")));
    assert!(!s.remove(Path::new("/a/c")));    // non-existent
    assert!(s.remove(Path::new("/a/b")));     // exists
    assert!(s.inner.is_empty());
    assert!(s.parents.is_empty());
}
```
Note: test code accesses private fields (`s.inner`, `s.stack`, `s.cursor`) directly because tests are in the same file.

**Error testing:**
```rust
// Testing absence (None/false) explicitly
assert_eq!(bs.shift_backward(), None);
assert!(!s.add(Path::new("/a/b")));   // returns false when conflict

// Testing error propagation
let result = risky_function();
assert!(result.is_err());
// OR via ? in Result-returning tests
```

## CI Integration

**Matrix:** Ubuntu, Windows, macOS (all via `cargo test --workspace --verbose`)
**Trigger:** push/PR to `main` branch
**Lint checks run separately:** Clippy (`cargo clippy --all`), rustfmt (`rustfmt +nightly --check **/*.rs`), StyLua
**Caching:** sccache via `mozilla-actions/sccache-action`

---

*Testing analysis: 2026-03-21*
