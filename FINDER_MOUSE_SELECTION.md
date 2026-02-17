# Finder-Style Mouse Selection

## What this does

This adds macOS Finder-style mouse selection to Yazi's file list:

- **Left-click** — moves cursor to the clicked file (unchanged behavior)
- **Ctrl-click** (or Cmd/Super-click) — toggles individual file selection
- **Shift-click** — range-selects all files between the last anchor point and the clicked file
- **Double-click** — opens the file or enters the directory
- **Right-click** — opens the "Open with..." interactive picker

The **anchor point** tracks the last file you explicitly interacted with. It's set on every click and reset when you navigate to a new directory. When entering a directory, the anchor defaults to whichever file is highlighted, so Shift-click works immediately without a prior click.

Selections are cleared automatically on directory change to prevent accidental operations on files you can no longer see.

## Why

Yazi already has excellent keyboard-driven selection (visual mode, Space to toggle), but mouse users have no way to select files without the keyboard. Every major file manager — Finder, Nautilus, Dolphin, Windows Explorer — supports modifier-click selection. This brings Yazi up to parity with those conventions.

The feature is entirely opt-in: if you don't use a mouse, nothing changes. All existing keyboard workflows are unaffected.

## What changed (3 files)

### 1. `yazi-binding/src/mouse.rs` — Expose modifier keys to Lua

crossterm's `MouseEvent` already carries a `modifiers: KeyModifiers` field, but it wasn't exposed to Lua. Added three boolean fields following the exact pattern of the existing `is_left`/`is_right`/`is_middle` fields:

```rust
fields.add_field_method_get("is_ctrl", |_, me| {
    Ok(me.modifiers.contains(KeyModifiers::CONTROL))
});
fields.add_field_method_get("is_shift", |_, me| {
    Ok(me.modifiers.contains(KeyModifiers::SHIFT))
});
fields.add_field_method_get("is_super", |_, me| {
    Ok(me.modifiers.contains(KeyModifiers::SUPER))
});
```

`KeyModifiers` is imported at the top level alongside the existing `MouseButton` import, matching the file's convention.

This is a general-purpose addition — any plugin or preset component can now read modifier state on mouse events.

### 2. `yazi-plugin/preset/components/current.lua` — Click handler

Replaced the minimal `Current:click` with a modifier-aware handler. Key decisions:

- **State lives on the class table** (`Current._anchor`, `Current._last_click`, `Current._cwd`), not on instances. This is consistent with how `Entity._inc` and `Entity._children` persist across render cycles — instances are recreated each frame via `Root:build()`, but class-level fields survive.

- **Batch selection uses `toggle_all`** with a URL array and `state = "on"`, the same pattern used by `fzf.lua` (line 32-33). No per-file toggle loop, no new Rust actors.

- **Double-click detection** uses `ya.time()` with a 0.4s threshold on the same row, consistent with how `mime-local.lua` and `folder.lua` use `ya.time()` for timing.

- **Both Ctrl and Super modifiers** are accepted for toggle-click. Super (Cmd) is the Finder convention on macOS, but many terminals intercept it. Ctrl is the universal fallback and matches Windows/Linux conventions.

- **Directory change detection** compares `tostring(self._folder.cwd)` against `Current._cwd` and resets the anchor + clears selections when it changes. This prevents stale anchors from a previous directory and avoids accidentally operating on invisible selections.

### 3. `yazi-plugin/preset/components/root.lua` — Dialog dismiss on click

Previously, `Root:click` returned early when `cx.layer ~= "mgr"` (i.e., when any dialog was open), swallowing all mouse events. Now it dismisses the dialog and falls through to normal click handling:

```lua
if tostring(cx.layer) ~= "mgr" then
    if up then return end
    ya.emit(tostring(cx.layer) .. ":close", {})
end
```

This uses the `layer:command` routing syntax from `Cmd::new` — the command name prefix selects the target layer. So `pick:close` dismisses the "Open with..." picker, `input:close` dismisses text inputs, `confirm:close` dismisses confirmations, etc. No hardcoded layer names.

The `up` guard prevents the mouse-up event from the same click that opened the dialog from immediately dismissing it.

The fall-through (no `return` after close) means right-clicking a different file while a dialog is open will: close the current dialog → reveal the new file → open the dialog for it — all in one click, matching Finder's behavior.

## Problems solved along the way

1. **Modifier keys weren't available in Lua** — crossterm provides them, but yazi-binding didn't expose them. A 9-line addition to `mouse.rs`.

2. **`ya.emit()` always routes to the Mgr layer** — discovered via `call.rs` line 11: `Some(Layer::Mgr)`. Initially tried `ya.emit("escape", {})` and `ya.emit("close", {})` to dismiss dialogs, both hit the wrong layer. The `layer:command` prefix syntax (from `Cmd::new`'s name parsing) solved it.

3. **Mouse-up from the same click dismissed the dialog** — right-click fires both Down (opens dialog) and Up (was dismissing it). Added an `up` guard so only Down events trigger close.

4. **Terminal intercepting Shift-click** — Shift-click is typically reserved for terminal text selection. Solved with XTSHIFTESCAPE escape sequences in the shell launcher wrapper:
   ```sh
   printf '\e[>1s'  # tell terminal to pass shift-clicks to application
   yazi "$@"
   printf '\e[>0s'  # restore terminal shift-click handling on exit
   ```

5. **Terminal intercepting right-click** — Ghostty (and many terminals) show a context menu on right-click, consuming the event before it reaches the application. Solved with a Ghostty config setting:
   ```
   right-click-action = ignore
   ```
   This passes right-click through to all terminal applications globally. Other terminals will have equivalent settings.

## Cross-platform compatibility

The implementation is fully cross-platform. No macOS-specific code.

- **Ctrl-click** — works in all terminals on all platforms (primary modifier)
- **Super/Cmd-click** — works where the terminal passes Super through (Ghostty, Kitty, iTerm2 on macOS; some Linux terminals)
- **Shift-click** — works where the terminal supports XTSHIFTESCAPE or doesn't intercept Shift-click for text selection. This is a terminal configuration concern, not a Yazi concern
- **Double-click and right-click** — standard mouse events, universally supported
- **Right-click passthrough** — requires terminal not intercepting it for a context menu (e.g., Ghostty's `right-click-action = ignore`)

Terminal-level configuration (XTSHIFTESCAPE, right-click passthrough) is external to Yazi and can be handled in wrapper scripts or terminal config. Yazi itself just reads the modifier flags that crossterm provides.

## What stays unchanged

- **entity.lua** — untouched; plain left-click and right-click delegate to `Entity:click` for the default behavior path
- **Rust selection system** — `Selected`, `toggle`, `toggle_all`, `visual_mode` all used as-is
- **No new Rust actors, structs, or event types**
- **Keyboard workflows** — visual mode, Space toggle, all keybindings work exactly as before
- **Scroll and touch handlers** — unchanged
