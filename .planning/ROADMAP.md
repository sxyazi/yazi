# Roadmap: Yazi Image Preview Quality Improvement

## Overview

Two phases deliver a surgical, single-file change to `yazi-adapter/src/image.rs`. Phase 1
implements linear-light-correct downscaling using `fast_image_resize`, covering all resize and
compatibility requirements in one coherent pass (the three correctness invariants — gamma
linearization, alpha premultiplication, filter config mapping — cannot be split without creating
partial-correctness windows). Phase 2 adds the unit test for linear-light correctness and
validates the implementation against the full image-type smoke test matrix.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Implementation** - Replace both resize call sites with linear-light-correct `fast_image_resize`, add dependency, map all config filter strings
- [ ] **Phase 2: Verification** - Add unit test for linear-light correctness and smoke-test across all image types and terminal protocols

## Phase Details

### Phase 1: Implementation
**Goal**: Image previews use linear-light-correct downscaling via `fast_image_resize` with all existing config options preserved
**Depends on**: Nothing (first phase)
**Requirements**: RESIZE-01, RESIZE-02, RESIZE-03, RESIZE-04, RESIZE-05, COMPAT-01, COMPAT-02, COMPAT-03, COMPAT-04
**Success Criteria** (what must be TRUE):
  1. Opening an image in yazi produces a visibly sharper preview than before (no blurring of fine detail at default lanczos3 filter)
  2. All existing `image_filter` config values (`nearest`, `triangle`, `catmull-rom`, `gaussian`, `lanczos3`) continue to work without error or silent degradation
  3. All terminal protocol drivers (KGP, KGP-old, IIP, Sixel, Chafa, Ueberzug) display images without regressions
  4. `precache()` public signature is unchanged — Lua plugins that call it continue to function
  5. `cargo build --release` succeeds and `cargo clippy --all` produces no new warnings
**Plans:** 1 plan

Plans:
- [ ] 01-01-PLAN.md — Replace resize pipeline with linear-light fast_image_resize (dependency + code + verification)

### Phase 2: Verification
**Goal**: The implementation is confirmed correct across all image types, protocols, and edge cases, with at least one automated correctness test
**Depends on**: Phase 1
**Requirements**: TEST-01
**Success Criteria** (what must be TRUE):
  1. A unit test exists that downscales a 50% gray sRGB patch to 1x1 and asserts the result is within tolerance of the linear-light-correct value (not the sRGB-space-incorrect midpoint)
  2. RGBA PNG with transparency displays without color fringing on transparent edges in IIP/WezTerm
  3. EXIF-rotated JPEG (phone photo) previews with correct orientation and no quality regression
  4. `cargo test --workspace` passes with the new test included
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Implementation | 0/1 | Planned | - |
| 2. Verification | 0/TBD | Not started | - |
