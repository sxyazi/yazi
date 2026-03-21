# Yazi Image Preview Quality Improvement

## What This Is

An improvement to yazi's image preview downscaling so that thumbnails displayed in the terminal preview pane are visibly sharper. The current pipeline uses the `image` crate's built-in `resize()` with configurable filter types, but the results appear noticeably less sharp than native image viewers like macOS Preview.

## Core Value

Image previews in yazi should look as sharp as the terminal's pixel resolution allows — downscaling artifacts should not be the bottleneck.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Improve the downscaling/resizing algorithm in `yazi-adapter` so preview thumbnails are visibly sharper
- [ ] Maintain existing configurability (users can still select filter types)
- [ ] No regression in preview latency perceptible during normal navigation

### Out of Scope

- Full image pipeline audit (ICC, encoding, protocol drivers) — focus is the resize step only
- Color accuracy improvements — not the reported issue
- Sixel/Chafa-specific quality — user's workflow is iTerm2/WezTerm (IIP protocol)
- Adding new image protocol support

## Context

**Current implementation:** `yazi-adapter/src/image.rs` — `Image::downscale()` and `Image::precache()` both call `img.resize(w, h, Self::filter())` using the `image` crate's `DynamicImage::resize()`. The default filter is `lanczos3` (configured in `yazi-config/preset/yazi-default.toml`).

**Why it's blurry:** The `image` crate's resize implementation, while functional, uses a simpler convolution approach compared to dedicated image processing libraries (e.g., `fast_image_resize`, `resize`). For downscaling large images to small preview sizes, the quality difference is noticeable — especially for fine detail and text in images.

**Key files:**
- `yazi-adapter/src/image.rs` — resize logic, filter selection, decode pipeline
- `yazi-config/src/preview/preview.rs` — `image_filter` and `image_quality` config
- `yazi-config/preset/yazi-default.toml` — default settings (`image_filter = "lanczos3"`, `image_quality = 90`)
- `yazi-adapter/src/drivers/iip.rs` — iTerm2 Inline Protocol driver (user's protocol)

**Brownfield context:** This is an existing, mature Rust project (1,363 commits, 15 workspace crates). The `yazi-adapter` crate has zero tests currently (noted in codebase concerns). The project uses Rust edition 2024 with MSRV 1.92.0.

## Constraints

- **Rust edition**: 2024, MSRV 1.92.0
- **Dependencies**: Must be compatible with existing workspace dependency versions
- **Performance**: Preview generation must not add perceptible latency
- **Backward compatibility**: Existing `image_filter` config option must continue to work
- **No tests exist**: `yazi-adapter` has zero tests — any changes are validated at runtime only

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Focus on resize algorithm only | User reports sharpness, not color; current pipeline is otherwise functional | — Pending |
| Target iTerm2/WezTerm (IIP) | User's primary terminal; IIP driver calls `Image::downscale()` directly | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-21 after initialization*
