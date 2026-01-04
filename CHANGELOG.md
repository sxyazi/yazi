# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/):

- `Added` for new features.
- `Changed` for changes in existing functionality.
- `Deprecated` for soon-to-be removed features.
- `Fixed` for any bug fixes.
- `Improved` for performance improvements.

## [Unreleased]

## [v26.1.4]

### Added

- Support VFS for preset previewers that rely on external commands ([#3477])
- Support 8-bit images in RGB, CIELAB, and GRAY color spaces ([#3358])

### Fixed

- `ya pkg` fails to write `package.toml` when the config directory does not exist ([#3482])
- A race condition generating unique filenames for concurrent file operations ([#3494])

## [v25.12.29]

### Added

- Remote file management ([#3396])
- Virtual file system ([#3034], [#3035], [#3094], [#3108], [#3187], [#3203])
- Shell formatting ([#3232])
- Multi-entry support for plugin system ([#3154])
- Zoom in or out of the preview image ([#2864])
- Improve the UX of the pick and input components ([#2906], [#2935])
- Show progress of each task in task manager ([#3121], [#3131], [#3134])
- New `fs.copy()` and `fs.rename()` APIs ([#3467])
- New experimental `ya.async()` API ([#3422])
- New `overall` option to set the overall background color ([#3317])
- Rounded corners for indicator bar ([#3419])
- New `bulk_rename` command always renames files with the editor ([#2984])
- `key-*` DDS events to allow changing or canceling user key events ([#3005], [#3037])
- New `--bg` specifying image background color in the preset SVG and ImageMagick previewers ([#3189])
- `filter` by full path (prefix + filename) in search view instead of just filename ([#2915])
- New `casefy` command for case conversion of the input content ([#3235])
- Allow dynamic adjustment of layout ratio via `rt.mgr.ratio` ([#2964])
- Support `.deb` packages ([#2807], [#3128], [#3209])
- Port several widespread GUI keys to the input component ([#2849])
- Support invalid UTF-8 paths throughout the codebase ([#2884], [#2889], [#2890], [#2895], [#3023], [#3290], [#3369])
- Allow upgrading only specific packages with `ya pkg` ([#2841])
- Respect the user's `image_filter` setting in the preset ImageMagick previewer ([#3286])
- New `duplicate` DDS event for copying files ([#3456])
- New `ind-sort` and `key-sort` DDS events to change sorting in Lua ([#3391])
- Allow custom mouse click behavior for individual files ([#2925])
- Display newlines in input as spaces to improve readability ([#2932])
- Fill in error messages if preview fails ([#2917], [#3383], [#3387])
- Search view shares file selection and yank state ([#2855])
- Offload mimetype fetching on opening files to the task scheduler ([#3141])
- Increase terminal response timeout to better tolerate slow SSH network environments ([#2843])

### Changed

- Rename `name` to `url` for open, fetchers, spotters, preloaders, previewers, filetype, and `globs` icon rules to support virtual file system ([#3034])
- Rename `mime` fetcher to `mime.local`, and introduce `mime.dir` fetcher to support folder MIME types ([#3222])
- Reclassify `hovered` and `preview_hovered` under `[mgr]` of `theme.toml` into `[indicator]` as `current` and `preview`, respectively ([#3419])
- Remove `$0` parameter in opener rules to make the `open` command work under empty directories ([#3226])
- Return `Path` instead of `Url` from `Url:strip_prefix()` and `File.link_to` to enforce type safety ([#3361], [#3385])
- Use `body` instead of the term `content` in confirmations ([#2921])
- Use `u16` instead of `u32` as the type of `max_width` and `max_height` options to avoid memory exhaustion ([#3313])
- Implement `__pairs` metamethod instead of `__index` for the callback argument of the `@yank` DDS event ([#2997])

### Deprecated

- Deprecate `$n`, `$@` (\*nix) and `%n`, `%*` (Windows) in `shell` command and opener rules in favor of new shell formatting ([#3232])
- Deprecate `ya.hide`, `ya.render`, and `ya.truncate` in favor of `ui.hide`, `ui.render`, and `ui.truncate` ([#2939])
- Deprecate `position` property of `ya.input()` in favor of `pos` to align with `ya.confirm()` and its type `ui.Pos` ([#2921])
- Deprecate `cx.tasks.progress` in favor of `cx.tasks.summary` ([#3131])
- Deprecate `frag` properly of `Url` in favor of `domain` ([#3034])
- Deprecate `ui.Rect.default` in favor of `ui.Rect {}` ([#2927])

### Fixed

- User-prepended open rules do not override presets ([#3360])
- Respect user's system media opener instead of hardcoding `mpv` ([#2959])
- Incorrect `$0` and `$@` parameters in `shell` command under empty directories ([#3225])
- Avoid appending a newline when reading clipboard contents ([#3059])
- Renew package `rev` only when it's empty ([#3200])
- Suspend only when there is a parent process ([#3008])
- Preserve open order for files with post-resolved MIME types ([#2931])
- A race condition in concurrent directory loading on a slow device ([#3271])
- Erase overlapping image portions when previewing errors ([#3067])
- Force Git checkout for plugin cache repositories ([#3169])
- Check compatibility when reusing previewer bytecode cache ([#3190])
- Disable kitty keyboard protocol on Windows due to `crossterm` inability to handle it ([#3250])
- Prevent quotes in file(1) arguments from being stripped under MSYS2 ([#3364])
- Expose `ya` CLI in the Snap build ([#2904])
- Fallback to `PollWatcher` for file changes watching on NetBSD ([#2941])
- Generate unique image IDs for Kgp to tolerate tmux ([#3038])

### Improved

- Make copy, cut, delete, link, hardlink, download, and upload tasks immediately cancellable ([#3429])
- Make preload tasks discardable ([#2875])
- Reduce file change event frequency ([#2820])
- Upload and download of a single file over SFTP in chunks concurrently ([#3393])
- Do not listen for file changes in inactive tabs ([#2958])
- Switch to a higher-performance hash algorithm ([#3083])
- Sequence-based rendering merge strategy ([#2861])
- Store only `Urn` instead of full `Url` in find results ([#2914])
- Zero-copy `UrlBuf` to `Url` conversion ([#3117])
- String interning to reduce memory usage of mimetype and URL domain ([#3084], [#3091])
- Do not pre-allocate memory for Lua tables ([#2879])
- Copy-on-write on command data, and avoid converting primitive types to strings thereby allocating memory ([#2862])
- Use `AnyUserData::type_id()` to reduce stack pushes ([#2834])
- App data instead of Lua registry to reduce stack pushes ([#2880])

## [v25.5.31]

### Fixed

- Expose `ui.Wrap` ([#2810])
- `forward --end-of-word` of the input should consider the mode's delta ([#2811])
- Make every effort to carry hidden states for dummy files ([#2814])

## [v25.5.28]

### Added

- Redesign tabs ([#2745])
- Support embedded cover for video preview ([#2640])
- Calculate real-time directory size in spotter ([#2695])
- Truncate long items in the file list ([#2754], [#2759], [#2778])
- Obscure input component for inputting passwords ([#2675])
- Improve path auto-completion results ([#2765])
- New `ya pkg` subcommand ([#2770])
- New `ya.emit()` API ([#2653])
- New `fs.calc_size()` API ([#2691])
- Allow custom exit code with `quit --code` ([#2609])
- New `--hovered` for the `copy` command ([#2709])
- `s` and `S` keybinds in the input component ([#2678])
- Limit memory usage for previewing large images ([#2602])
- Show error when image preview fails ([#2706])
- New `ui.Align`, `ui.Wrap`, and `ui.Edge` ([#2802])
- Make `ui.Line` renderable ([#2743])
- Checks in `ya pub` and `ya emit` subcommands to verify receiver exists and has necessary abilities ([#2696])
- Make the hover state for `reveal`, `sort`, and `hidden` commands stable ([#2657])
- New `--no-dummy` option for `reveal` command ([#2664])
- Fall back to `CSI 16 t` when PowerShell OpenSSH returns a fake terminal size ([#2636])

### Changed

- Deprecate `[manager]` in favor of `[mgr]` to make it consistent with other APIs ([#2803])
- Remove `tab_width` as it no longer needs to be set manually ([#2745])
- Move `tab_active` and `tab_inactive` to a dedicated `[tabs]` section ([#2745])
- Remove `sixel_fraction` as it's no longer needed ([#2707])

### Deprecated

- Deprecate `ya.mgr_emit()`, `ya.app_emit()` and `ya.input_emit()` ([#2653])
- Deprecate `ya.preview_widgets()` ([#2706])
- Deprecate the `Command:args()` method ([#2752])
- Deprecate the `ya pack` subcommand in favor of `ya pkg` ([#2770])
- Deprecate `LEFT`, `CENTER`, and `RIGHT` on `ui.Line` and `ui.Text` in favor of `ui.Align` ([#2802])
- Deprecate `NONE`, `TOP`, `RIGHT`, `BOTTOM`, `LEFT`, and `ALL` on `ui.Bar` and `ui.Border` in favor of `ui.Edge` ([#2802])
- Deprecate `WRAP_NO`, `WRAP` and `WRAP_TRIM` on `ui.Text` in favor of `ui.Wrap` ([#2802])

### Fixed

- Respect the user's `max_width` setting in the preset video previewer ([#2560])
- Reverse the mixing order of theme and flavor configuration ([#2594])
- No title is set when starts the first time ([#2700])
- `ya pub-to 0` checks if any peer is able to receive the message ([#2697])
- Detach background and orphan processes from the controlling terminal with `setsid()` ([#2723])
- Always try to create state directory before draining DDS data ([#2769])
- Avoid tmux interfering with kitty graphical sequences ([#2734])

### Improved

- Double directory size calculation speed ([#2683])
- 9x faster Sixel image preview ([#2707])
- Remove intermediate variables in natural sorting algorithm to avoid unnecessary allocation ([#2764])
- Avoid unnecessary memory allocation in `ya.truncate()` ([#2753])

## [v25.4.8]

### Added

- Enhance `fzf` integration ([#2553])
- Platform-specific key binding ([#2526])
- Custom search engine Lua API ([#2452])
- New `follow` command to follow files pointed to by symlinks ([#2543])
- Allow `tab_swap` to cycle tabs ([#2456])
- Show error message when directory fails to load ([#2527])
- New `symlink_target` to style the target of symbolic links ([#2522])
- Use Yazi in Helix directly without Zellij or tmux ([#2461])
- New `<C-A>` and `<C-E>` keybindings to select entire line for the input component ([#2439])
- New `fs.expand_url()` API ([#2476])
- New `ui.Text:scroll()` API for setting text to scroll horizontally or vertically ([#2589])
- Allow initializing input when opening it with commands like `rename`, `create`, `find`, `filter`, etc. ([#2578])
- New `@sync peek` annotation for sync previewers ([#2487])
- New `ya.id("app")` to get `YAZI_ID` in plugins ([#2503])
- New `base` field for the `Url` ([#2492])
- New `rt.term` exports terminal emulator information ([#2442])
- Allow bulk renaming to include trailing content in addition to the required new names ([#2494])
- Log `tmux` call execution time to logs ([#2444])

### Changed

- Navigation wraparound with new `arrow prev` and `arrow next` commands ([#2485], [#2540])
- Swap default key bindings for fzf and zoxide ([#2546])
- Switch to `resvg` as the SVG renderer ([#2581])
- Make `frag`, `name`, `stem`, `ext`, and `parent` on `Url`, `name` on `tab::Tab`, and `is_hovered` on `fs::File` properties ([#2572])
- Replace `tasks_show` and `close_input` with `tasks:show` and `input:close` ([#2530])
- Replace `sync = true` with the `@sync peek` annotation ([#2487])

### Deprecated

- Deprecate `ui.Padding` and `ui.Rect:padding()` ([#2574])

### Fixed

- Always show the size in the status bar even in empty directories ([#2449])
- Remove the temporary extraction directory forcefully ([#2458])
- Align the behavior of the end-of-options marker (`--`) with that of the shell ([#2431])
- Respect hidden status of directory junctions and symlinks themselves on Windows ([#2471])

### Improved

- Rewrite config parser to double the startup speed ([#2508])
- Lazy compile and cache lua plugins as binary bytecode ([#2490])
- Faster image preview with optimized `magick` arguments ([#2533])
- Cache UserData fields ([#2572])

## [v25.3.2]

### Added

- Expose all theme fields in Lua ([#2405])
- Expose almost the entirety of the user's configuration in Lua ([#2413])

### Fixed

- `STDIN_FILENO` poll always returns 0 under SSH ([#2427])
- Ignore stdin redirection to ensure always accessing the real tty ([#2425])
- Incorrect deprecation warning when the plugin doesn't exist ([#2418])

## [v25.2.26]

### Added

- Allow to specify layer for keymap commands ([#2399])
- New `rt` and `th` allow to access user configuration and theme scheme in sync/async plugins consistently ([#2389], [#2392], [#2393], [#2397])
- New `tbl_col` and `tbl_cell` in theme system for spotter table styling ([#2391])
- Allow different separators to be applied individually to the left and right sides of the status bar ([#2313])
- `ripgrep-all` support for the `search` command ([#2383])
- Respect the user's `max_width` setting in the preset PDF preloader ([#2331])
- Respect the user's `wrap` setting in the preset JSON previewer ([#2337])
- Respect the user's `image_alloc` setting in the preset ImageMagick previewer ([#2403])
- New `external` and `removable` fields in the `fs.partitions()` API ([#2343])
- CSI-based Vim and Neovim built-in terminal detection for better accuracy ([#2327])

### Changed

- Replace `separator_open` and `separator_close` with `sep_left` and `sep_right` ([#2313])
- Rename the `[completion]` component to `[cmp]` ([#2399])

### Deprecated

- Deprecate `MANAGER`, `PREVIEW`, `PLUGIN`, and `THEME` in favor of `rt` and `th` ([#2389])
- Deprecate `ya.manager_emit()` in favor of `ya.mgr_emit()` ([#2397])

### Fixed

- Didn't reset previous `Cha` when loading directories in chunks ([#2366])
- Load mount points with the best effort even if the `/dev/disk/by-label` directory does not exist ([#2326])
- Add maximum preview limit under `/proc` virtual file system ([#2355])

## [v25.2.11]

### Added

- New `overall` option under `[status]` to allow specifying the overall style of the status bar ([#2321])
- Reduce terminal response wait timeout ([#2314])

### Fixed

- Unable to delete sealed files on Windows due to platform differences ([#2319])
- Reverse the order of CSI-based and environment-based terminal detection ([#2310])

## [v25.2.7]

### Added

- Mount manager ([#2199])
- New `ya.confirm()` API ([#2095])
- New `arrow top` and `arrow bot` commands to jump to the top and bottom ([#2294])
- Support end of options (`--`) marker for all commands ([#2298])
- Replace mode and Vim motions (`W`, `E`, `B`, `^`, `_`) for inputs ([#2143])
- New `ya pack -d` subcommand to delete packages ([#2181])
- `ya pack` supports adding and deleting multiple packages at once ([#2257])
- Theme support for the spotter border and title ([#2002])
- Use positional argument instead of `--args` for the `plugin` command ([#2299])
- Support and hide Windows system files by default ([#2149])
- New `--no-cwd-file` option for the `close` command ([#2185])
- Prompt users missing fzf in the zoxide plugin ([#2122])
- More decent package locking mechanism ([#2168])
- Custom modal component API ([#2205])
- Support local `tmux` image preview over SSH
- New `@since` plugin annotation to specify the minimum supported Yazi version ([#2290])
- Allow preloaders to return an optional `Error` to describe the failure ([#2253])
- ARM64 Snap package ([#2188])
- Support `package.toml` as a symlink ([#2245])
- New `cx.layer` API to determine the current UI layer ([#2247])
- Channel and multi-concurrent task join support for the plugin system ([#2210])
- Support `application/mbox` mimetype ([#2173])
- `cbr` and `cbz` as valid archive extensions ([#2077])

### Deprecated

- Deprecate `--args` in the `plugin` command in favor of a 2nd positional parameter ([#2299])
- Deprecate plugin entry file `init.lua` in favor of `main.lua` ([#2168])
- Deprecate `arrow -99999999` and `arrow 99999999` in favor of `arrow top` and `arrow bot` ([#2294])
- Deprecate the numeric return value of preloaders in favor of a boolean return value ([#2253])
- Deprecate `ya.md5()` in favor of `ya.hash()` ([#2168])

### Fixed

- `before_ext` should exclude directories since only files have extensions ([#2132])
- Element style of `ui.Text` was not applied to the entire area ([#2093])
- Incorrect monorepo sub-plugin path resolution ([#2186])
- Use `u32` for parsing Linux partition blocks ([#2234])
- Unmangle the hexadecimal space strings (`"\x20"`) in Linux partition labels ([#2233])
- JSON value `null` should be deserialized as Lua `nil`, not lightweight userdata `null` ([#2242])
- Don't check if has a hovered file in advance, only do so when `--hovered` is explicitly specified ([#2105])
- Handle broken pipe errors gracefully ([#2110])

### Improved

- Detach the watch registration from the main thread ([#2224])

## [v0.4.2]

### Added

- More supported archive formats to the preset config ([#1926])
- New `fs.create()` Lua API ([#2068])
- New `--cwd` parameter for the `shell` command and `fs.cwd()` API ([#2060])
- Allow `noop` for single-key chords by removing the mixing length limit ([#2064])
- Support for Android platform in the `for` qualifier of opener ([#2041])

### Fixed

- Set the current working directory in a thread-safe way ([#2043])
- Interactive `cd` autocomplete doesn't follow the latest `CWD` changes ([#2025])
- Offset cursor shift when deleting multiple files in bulk ([#2030])
- Missing a render after resuming from an external blocking process ([#2071])
- Missing a hover after reordering from an external plugin ([#2072])
- Use a less intrusive `DSR` instead of `DA1` workaround to forward terminal responses twice in tmux ([#2058])
- `allow-passthrough` must be set to `on` to prevent `tmux` from forwarding the real terminal's response to the inactive pane ([#2052])

## [v0.4.1]

### Fixed

- Correctly handle CRLF on Windows in preset `archive` and `json` plugins ([#2017])
- Failed to parse certain image dimensions for Überzug++ backend ([#2020])
- Disable passthrough when the user launches Yazi in Neovim inside tmux ([#2014])

## [v0.4.0]

### Added

- Spotter ([#1802])
- Support transparent image preview ([#1556])
- Auto switch between dark and light icons/flavors based on terminal backgrounds ([#1946])
- Allow disabling certain preset keybinds with the new `noop` virtual command ([#1882])
- New `ya emit` and `ya emit-to` subcommands to emit commands to a specified instance for execution ([#1979])
- Custom styles for the `confirm` component ([#1789])
- Make the builtin `extract` plugin support compressed tarballs (`*.tar.gz`, `*.tar.bz2`, etc.) ([#1583])
- Launch from preset settings if the user's config cannot be parsed ([#1832])
- Prioritize paths that need to be processed first during bulk renaming ([#1801])
- New `copy --separator` option to allow specifying the path separator ([#1877])
- Set a different input title for `create --dir` ([#1650])
- Include package revision hash in `ya pack --list` ([#1884])
- New `load` DDS event ([#1980])
- New log system ([#1945])
- New `ui.Text` and `ui.Table` layout elements ([#1776])
- Support passing arguments to previewer/preloader/spotter/fetcher ([#1966])
- Move notification from top-right to bottom-right corner to avoid covering content as much as possible ([#1984])
- Append the suffix to the end instead of start when generating unique filenames for directories ([#1784])
- Allow overriding and rewriting the sync methods of built-in plugins ([#1695])
- Fallback to `CSI 16 t` for certain terminals that do not support `TIOCGWINSZ` ([#2004])
- Support calling methods in built-in plugins with arbitrary types of arguments (`self` can now be omitted) ([#1666])
- Support `assets` installation for the `ya pack` subcommand ([#1973])
- Complete and consistent support for the `ui.Style()` API
- Image ICC profiles for better color accuracy ([#1808])
- Support reading non-UTF8 data with `Child:read_line()` API ([#1816])
- New `area()` method for renderable elements ([#1667])
- `yazi --debug` supports detecting whether `tmux` is built with `--enable-sixel` ([#1762])

### Changed

- Eliminate the `x-` prefix in MIME types ([#1927])
- Remove the `vnd.` prefix from mimetype to solve differences introduced in the newest `file(1)` v5.46 ([#1995])
- Rename the term `select` to `toggle` to reserve `select` for future use ([#1773])
- Correct the misuse of the term `ctime` and unify others ([#1761])
- Replace `ffmpegthumbnailer` with `ffmpeg` as the video preview backend to support spotter ([#1928])
- Use an `Error` userdata instead of a plain error code for I/O errors ([#1939])
- Remove `ui.ListItem` since it's no longer necessary ([#1772])
- Decouple coordinates from `ui.List`, `ui.Bar`, `ui.Border`, and `ui.Gauge` ([#1782])
- Make `backspace` command not close the input even when value is empty ([#1680])
- Remove the meaningless `--confirm` option to simplify the `shell` command ([#1982])
- Use `dark` and `light` instead of `use` under `[flavor]` to support auto-switching between light and dark modes ([#1946])
- Unify the `fg_dark` and `fg_light` into one `fg` since `fg_light` is redundant and never used ([#1946])
- Extend the available styles for `mode` by separating `mode` from the `separator` styles ([#1953])

### Deprecated

- Deprecate `--sync` option for the `plugin` command ([#1891])
- Deprecate `ui.Paragraph` in favor of `ui.Text` ([#1776])
- Deprecate the task info of `peek()`, `seek()`, and `preload()` from `self` in favor of a `job` parameter ([#1966])
- Deprecate parameter list of `entry()` from its first argument in favor of a `job` parameter ([#1966])
- Deprecate the number of units of `seek()` from its first argument in favor of a `job` parameter ([#1966])

### Fixed

- Introduce a new `btime` term to align `ctime` with Unix ([#1761])
- Match icon by extension case-insensitively ([#1614])
- Copy the CWD path with `c => d` regardless even if the directory is empty ([#1849])
- Respect the `image_quality` setting in preset PDF previewer ([#2006])
- Images were not cleared when closing a tab in front of the current tab ([#1792])
- Replace control characters to printable characters in plain text preview ([#1704])
- One file's MIME type changed multiple times without triggering a preview again ([#1682])
- Reset image rendering and skip peeking if the TUI in the background ([#1833])
- File upserting should handle deleting in edge cases where the source and target URNs are different ([#1737])
- Revise `revision` if the new file list is empty but the previous one was not ([#2003])
- Update `rustix` to fix the `enable_raw_mode()` error on WSL/Android

### Improved

- Merge multiple file operations into one to greatly speed up updates in large directories ([#1745])
- Eliminate all memory reallocations during sorting ([#1846])
- Introduce URN to speed up large directory sorting, updating, locating ([#1622], [#1652], [#1648])
- Improve jemalloc memory allocator efficiency ([#1689])
- Lazy load `ui`, `ya`, `fs`, and `ps` ([#1903])
- Avoid unnecessary allocations in base64 encoding of inline image protocol ([#1639])
- Introduce copy-on-write for event system to eliminate all memory reallocations ([#1962])
- Apply rotate in place to images with orientation ([#1807])
- Introduce reflow for the rendering engine ([#1863])

## [v0.3.3]

### Added

- `size` linemode supports showing the file count for directories ([#1591])
- Support image preview in Windows Terminal ([#1588])
- Add `is_absolute`, `has_root`, `starts_with`, `ends_with`, `strip_prefix` to `Url` ([#1605])

### Fixed

- Keybindings disappear when mixing presets with a wrong filter condition ([#1568])
- Squeeze `offset` of the file list after resizing window ([#1500])
- Check compositor support status before using ueberzug wayland output ([#1566])
- Fallback to `PollWatcher` for file changes watching on WSL ([#1574])

### Improved

- Truncate long lists in confirm dialogs ([#1590])

## [v0.3.2]

### Added

- New confirm component ([#1167])
- Word wrapping in `code` previewer ([#1159])
- New `--dir` option for `create` command ([#1505])
- New `ext()` method for `Url` ([#1528])
- Make the builtin `code` previewer handle invalid carriage return chars and binary streams better ([#1550])

### Fixed

- Wait till mimetype is resolved to avoid preview flickering ([#1542])
- Use a different cache directory for each user to avoid permission issues ([#1541])
- Filter out candidates that overlap with longer key chords from the which component ([#1562])
- Overlong single-line text preview containing escape sequences was not being properly escaped ([#1497])

### Improved

- New `image_delay` option debounces image previews to avoid lag caused by terminal image decoding during fast scrolling ([#1512])
- Only scan the first 1024 bytes to detect if it's binary, apply `\r` fixes only to content within the visible range, avoid unnecessary allocations during natural sorting ([#1551])

## [v0.3.1]

### Added

- Start with multiple tabs with different paths ([#1443])
- Key notion shorthands such as `<C-S-x>` as `<C-X>` ([#1448])
- Support `F13` - `F19` keys ([#1446])
- New `--cursor` for the `shell` command ([#1422])
- New `search_do` command to make it easier to achieve a flat view ([#1431])
- Portrait orientation preview for EXIF image ([#1412])
- Keybinding for the `hardlink` command ([#1461])
- New `empty` previewer for empty and `/proc/*` files ([#1482])
- Note about filtering in the help menu ([#1361])
- New `tab` DDS event on tab switch ([#1474])
- New `status()` method for `Command` ([#1473])

### Fixed

- Directory loading status ([#1439])
- Resolve relative path when expanding path ([#1428])
- DDS static messages only work when at least two instances are running ([#1467])
- Escape files containing special `\x1b` characters and render it as plain text ([#1395])
- 7zip shows different error messages for wrong password ([#1451])
- 7zip shows different error messages for RAR and ZIP files ([#1468])
- Newly created directories with the same name causing a false positive in directory loading optimization due to having the same modification time ([#1434])
- Close stdin before waiting for child process ([#1464])

## [v0.3.0]

### Added

- Package manager ([#985], [#1110])
- Support mouse event ([#1038], [#1139], [#1232])
- New `extract` built-in plugin for archive extracting ([#1321])
- Redesign icons ([#1086])
- Font preview ([#1048])
- SVG, HEIC, AVIF, and JPEG XL preview support ([#1050], [#1249])
- Simplify keybindings ([#1241])
- New command `hardlink` to create hard links ([#1268])
- Keep file creation time on macOS and Windows ([#1169])
- Sort randomly ([#1291])
- New linemode to show file ownership ([#1238])
- New linemode to show file ctime ([#1295])
- New `--hovered` option for the `rename` and `remove` commands ([#1227])
- Support Super/Command/Windows key with `D-` notation ([#1069])
- Interactive `cd` path auto-completion supports `~` expansion ([#1081])
- Preview files containing non-UTF-8 characters ([#958])
- Expand Windows paths like "D:" that only have a drive letter but no root ([#948])
- Close confirmation dialogs and exit automatically when the ongoing task gone ([#997])
- Case-insensitive special keys in keymappings ([#1082])
- Transliteration for natural sorting ([#1053])
- New `ya.clipboard()` API ([#980])
- New `debounce` option for the `ya.input()` API ([#1025])
- Support `yazi-cli` for Nix flake ([#944])
- Support `stdin` and pipe for `Child` API ([#1033])
- New `ya sub` subcommand to subscribe to DDS events ([#1004])
- Allow specifying `$YAZI_ID` with a command-line argument ([#1305])
- DDS client-server version check ([#1111])
- New `bulk` DDS event ([#937])
- Support `cargo binstall yazi-fm` and `cargo binstall yazi-cli` ([#1003])
- Show `ya` CLI version in the `yazi --debug` output ([#1005])
- Detect terminal type in tmux with CSI sequence in passthrough mode ([#977])

### Changed

- Use Ctrl+c instead of Ctrl+q as the universal close key to follow the conventions
- Replace Alt+k/Alt+j with K/J as the `seek` keybindings to avoid Alt key not working in certain terminals
- Replace Ctrl+Enter with Shift+Enter as the alternative key for Shift+o so that it corresponds to Enter being `o` (without Shift)
- keep original state of `sort` command in favor of specifying `yes` or `no` to explicitly apply a new state to its `--reverse`, `--dir-first`, and `--translit`
- Move `mime` plugin from `[plugin.preloaders]` to `[plugin.fetchers]` of yazi.toml
- Turn `success` and `code` into properties of `Status` instead of methods
- Remove `fs.cha_follow(url)` in favor of `fs.cha(url, true)`
- Rename `is_block_device`, `is_char_device`, and `is_socket` of `Cha` to `is_block`, `is_char`, and `is_sock` for simplicity

### Fixed

- Different filenames should be treated as the same file on case-insensitive file systems ([#1151])
- Suppress warnings for different name representations of the same file in the case-insensitive file system when renaming ([#1185])
- Avoid duplicate candidates in the `which` component ([#975])
- Sixel support from certain `st` forks cannot be detected ([#1094])
- Move the DDS socket file out of the cache directory to avoid being affected by `yazi --clear-cache`
- Build `jemalloc` with 64KB pagesize on linux/arm64 ([#1270])
- Cursor gets out of sync occasionally at image previewing through IIP under tmux ([#1070])

### Improved

- Reimplement and significantly speed up archive previewing ([#1220])

## [v0.2.5]

### Added

- Data distribution service ([#826], [#855], [#861], [#867], [#868], [#871], [#880], [#895], [#913], [#928], [#933], [#940])
- Re-implement fzf and zoxide as built-in plugins ([#884], [#881])
- Preserve files' modified at timestamp while copying ([#926])
- New `--orphan` option for `shell` command ([#887])
- Smart-case for completion of interactive `cd` paths ([#910])
- Allow creating a tab with the startup directory through `tab_create` without specifying a path ([#917])
- Bunch of new debugging information to `yazi --debug` ([#824])
- Time-based selection order preservation ([#843])
- Placeholder message when there are no files in the list ([#900])
- Enhance `ya.dbg()` and `ya.err()` by support arbitrary types ([#835])
- Trigger path completion with both `/` and `\` on Windows ([#909])
- Allow opening interactively with the `--chosen-file` flag ([#920])
- Support `YAZI_FILE_ONE` in the preset `file` previewer ([#846])

### Deprecated

- Deprecate the `jump` command in favor of `plugin fzf` and `plugin zoxide` ([#884], [#881])

### Fixed

- Kill all spawned processes on exit ([#812])
- Prevent pasting a directory into itself ([#925])
- Use `BTreeSet` for selected files to maintain order ([#799])
- CJK text rendering issue where the input popup component overlaps with images ([#879])

### Improved

- Accelerate kitty graphics protocol encoding by avoiding string reallocation ([#837])
- Wrap stderr with `BufWriter` to avoid frequent system calls thereby increase rendering frame rate ([#849])
- Switch to `globset` to reduce CPU time spent on matching icons ([#908])
- Re-implement file watcher in an async way ([#877])
- Cache each file's icon to avoid redundant calculations at rendering ([#931])
- Port `require()` and `ya.sync()` to Rust to avoid plugin information initialization ([#853])

## [v0.2.4]

### Added

- Vim-like notification with new `ya.notify()` API ([#659], [#749], [#780])
- New `ya.input()` API to request user input ([#762])
- Cross-directory selection ([#693])
- Colorize the icons ([#683])
- Flavors ([#753])
- New counter component shows the number of yanked/selected items ([#646])
- New `scrolloff` option to keep a margin when scrolling ([#679])
- New `<Home>`, `<End>`, and `<Delete>` keybindings for inputs ([#665])
- New `<C-p>` and `<C-n>` for the select component to move the cursor up/down ([#779])
- New `Ctrl+[` as an alternative to the escape key ([#763])
- New option `--hovered` for the `open` command allows only to open the currently hovered file ([#687])
- Support musl builds for Linux ([#759])
- New `--debug` flag to print debug information ([#794])
- Send a foreground notification to the user when the process fails to run ([#775])
- Nested conflict detection for cross-directory selections ([#689])
- New `prepend_rules` and `append_rules` for `[open]` and `[icon]` ([#754], [#670])
- Call sync functions within an async plugin ([#649])
- Allow access to complete app data from all tabs ([#644])
- Ability to sort candidates in the which component ([#662])
- Expose selected/yanked files as Lua API ([#674])
- New `cx.yanked` API to access yanked files ([#788])
- New `$0` (Unix) / `%0` (Windows) to access the hovered file in `shell` command ([#738])
- New `ya.hide()` API to hide the UI temporarily ([#792])
- Allow both `/` and `\` for folder creation on Windows ([#751])
- New `parse()` method for the line elements to parse ANSI sequences
- New `ui.Clear` component to clear areas ([#786])
- Support `YAZI_FILE_ONE` environment variable for `file(1)` path ([#752])
- Merge wildcard preloader and previewer rules via `append_preloaders` and `append_previewers`

### Deprecated

- Deprecate the `exec` property in yazi.toml, keymap.toml, and theme.toml in favor of `run`

### Fixed

- Rendering fails when no file type style is matched ([#721])

### Improved

- Cache loaded plugins ([#710])
- Cheaper sync context initialization ([#643])
- Prefer `raw_get()` and `raw_set()`

## [v0.2.3]

### Added

- Preview image over SSH ([#585])
- New `unyank` command ([#313])
- Customize number of columns of the which component ([#571])
- Support passing arguments to plugin ([#587])
- New `image_quality` and `sixel_fraction` options to configure the image preview quality ([#576])
- New `ya.which()` API for custom key events ([#617])
- New `ya.quote()` API to quote strings safely
- `plugin` command for each layer
- Plugin-specific state persistence ([#590])
- Allow to configure image filter ([#586])
- Shorten unit names and add more units to `ya.readable_size()`
- Support char device in `[filetype]` ([#628])
- File hidden attributes on Windows ([#632])
- Make `trash` crate optional on Android ([#600])

### Fixed

- Parent folder not tracking CWD ([#581])
- Input offset is not reset when renaming with `--cursor=start` and the filename is too long ([#575])

### Improved

- Read directory in bulk in the background at startup ([#599])
- Lazy sorting when loading large directories to reduce CPU consumption ([#607])

## [v0.2.2]

### Added

- `prepend_keymap` and `append_keymap` for configuration mixing ([#546])
- `file(1)` as the file fallback previewer ([#543])
- Submit both completion and input with a single press of enter ([#565])
- Allow the spawned child processes to suspend ([#556])
- New `ya.host_name()` API ([#550])
- Desktop entry and logo ([#534])
- Snap package ([#531])
- Support Windows ARM64 ([#558])
- Image preview in Tabby terminal ([#569])

### Fixed

- Can't display file name with invalid UTF-8 ([#529])

### Improved

- New event system allows multiple commands to reuse a single render ([#561])

## [v0.2.1]

### Fixed

- Renaming may cause a crash when encountering Unicode characters ([#519])

## [v0.2.0]

### Added

- New `filter` command to filter files on the fly ([#454])
- Sort by file extension ([#405])
- Custom preloader and previewer ([#401])
- New `plugin` command to run Lua plugins
- Auto-completion for input component ([#324], [#353], [#352])
- Start with the specified file hovers over ([#358])
- Emacs readline keybindings for inputs ([#345], [#382])
- New `--empty` and `--cursor` options for the `rename` command ([#513])
- New `--follow` option for `paste` command ([#436])
- Make `copy` command work over SSH with OSC 52 ([#447])
- New `reveal` command ([#341])
- Support colored icons ([#503])
- Support highlighting specific file types ([#510])
- Make the position of input and select components customizable ([#361])
- New `prepend_preloaders`, `append_preloaders`, `prepend_previewers`, `append_previewers` options for configuration mixing
- Cursor and page key navigation parity with Vim bindings ([#386])
- Use terminal ANSI colors for code highlighting by default
- New `image_alloc` and `image_bound` options to control image preview memory usage ([#376])
- New `suppress_preload` option to hide preload tasks ([#430])
- New kitty graphics protocol implementation for better compatibility with `tmux` through Unicode placeholders ([#365])
- New `ya.user_name()` and `ya.group_name()` API ([#469])
- New `ya.render()` to trigger a UI render
- Image orientation support ([#488])
- Raise open file descriptors limit at startup ([#342])
- Support image preview on WSL ([#315])
- Fine-grained scheduling priority ([#462])
- New `YAZI_LEVEL` environment variable to indicate the nested level ([#514])
- New `QuadrantInside` and `QuadrantOutside` border type

### Changed

- Rename the option `layout` to `ratio` to make it more self-explanatory
- Rename the `peek` command to `seek` to better convey the action of "seeking for" content to preview
- Rename the `--dir_first` option of `sort` command to `--dir-first` to make it consistent with the style of other commands
- Replace `[plugins.preload]` with the `init.lua` entry file

### Fixed

- `jq` previews empty when the user sets `tab_size=8` ([#320])
- Precache n-1 and n+1 pages ([#349])
- Popup components being covered by previewed images ([#360])
- Rust panics instead of returning an error when file times are invalid ([#357])
- Clear Sixel image with empty characters instead of `\x2B[K` to be compatible with GNOME VTE ([#309])
- Use `WAYLAND_DISPLAY` and `DISPLAY` to detect Wayland/X11 when `XDG_SESSION_TYPE` is not set ([#312])

### Improved

- Chunk loading for MIME types ([#467])
- Fallback to plain highlighter for long text ([#329])
- Reduce peak memory footprint during decoding large images ([#375])
- Clear only necessary cells when hiding images ([#369])
- New UI rendering architecture ([#468])
- Partial rendering progress and composite into a complete UI to reduce CPU consumption caused by frequent progress updates ([#509])

## [v0.1.5]

### Added

- New `find` command to find files ([#104])
- Linemode to show extra file info ([#291])
- New `sort_sensitive` option ([#155])
- Cross-platform opener rules ([#289])
- Multiple openers for a single open rule ([#154])
- Vim-like `gg`, `G` in the preset key mappings for boundary jumps
- Theme system ([#161])
- New `--force` option for `remove`, `create`, `rename` commands ([#179], [#208])
- Image preview within tmux ([#147])
- New `link` command that creates symlinks to the yanked files ([#167])
- New `orphan` option for opener rules to detach processes from the task scheduler ([#216])
- New `backward` and `forward` commands
- New `--smart` option for the `find` command to support smart case ([#240])
- Sorting for each tab individually ([#131])
- Suspend process with `Ctrl+z` ([#120])
- Percentage values for the `arrow` command to scroll half/full page (with newly added Vi-like `<C-u>`, `<C-d>`, `<C-b>`, and `<C-f>` keybindings) ([#213])
- Show keywords when in search mode ([#152])
- Tab switch wraparound ([#160])
- Highlight matched keywords in find mode ([#211])
- Customizable main UI border styles ([#278])
- `<BackTab>` key notion ([#209])
- Use of environment variables in `cd` paths ([#241])
- Nix Flakes package ([#205])
- New `V`, `D`, `C` Vim-like keybindings for the input component
- New `--no-cwd-file` option for the `quit` command to exit without writing the CWD file ([#245])
- Fallback to built-in code highlighting if `jq` is not installed ([#151])
- New `realtime` option for the input to support real-time input feedback ([#127])
- RGBA-16 image preview ([#250])
- FreeBSD and NetBSD support ([#169], [#178])
- Trash files on NetBSD ([#251])
- Image preview support on Mintty (Git Bash) terminal

### Changed

- Make glob expressions case-insensitive by default (with new `\s` for sensitive) ([#156])
- Make help items filtering case-insensitive

### Fixed

- `show_hidden` not properly applied to hovered folder ([#124])
- Notification of file changes in linked directories ([#121])
- Restore default cursor style when closing input from insert mode
- Task manager cursor position not reset after task cancellation
- Redirect clipboard process' stderr to /dev/null
- Delegate the `SIGINT` signal of processes with `orphan=true` to their parent ([#290])
- Inconsistent `Shift` key behavior on Unix and Windows ([#174])

### Improved

- Load large folders in chunks ([#117])
- Reimplemented natural sorting algorithm for ~6x faster case-insensitive sorting
- Kill process immediately after getting enough JSON or archive preview content to avoid wasting CPU resources ([#128])

## [v0.1.4]

### Added

- Help menu ([#93])
- Scrollable preview ([#86])
- Natural sorting ([#82])
- Windows support
- New `copy` command to copy file paths to clipboard ([#72])
- File chooser mode ([#69])
- Show symlink path ([#67])
- Respect `$EDITOR` environment variable when opening text files ([#91])
- Customizable main UI layout ratio ([#76])
- Allow accessing selected files when running shell commands ([#73])
- Update MIME type when file changes are detected ([#78])
- More clipboard backend: `xclip` and `xsel`, and Windows ([#74], [#75])
- New `cache_dir` option ([#96])
- New `YAZI_CONFIG_HOME` to specify the configuration directory ([#97])
- Black Box terminal image preview support ([#99])

### Deprecated

- Deprecate `--cwd` in favor of the positional argument ([#100])

### Fixed

- Make file(1) follow symbolic links when fetching file MIME type ([#77])
- Wrong height of the select component ([#65])
- Regression causing UI tearing when previewing images
- Specify `doNotMoveCursor` to make WezTerm render images sensibly

## [v0.1.3]

### Added

- Bulk rename ([#50])
- PDF preview and precache ([#18])
- New `sort_dir_first` option ([#49])
- Code highlighting supports more languages ([#22])
- Change the shell CWD on exit with the shell wrapper ([#40])
- Allow customizing the display name of openers ([#31])
- New `shell` command ([#24])
- Command template support for the `shell` command ([#48])
- Interactive `cd` ([#43])
- Show the output of running tasks in real time ([#17])
- Allow using the current directory name as tab name ([#41])
- Custom status bar separator ([#30])
- Fallback for opening files when no openers are available
- Preview files with `inode/empty` and `application/json` MIME types
- Transparent image support for the Sixel backend ([#14])
- Refresh image preview after terminal restoration
- New `micro_workers`, `macro_workers`, and `bizarre_retry` options to control task concurrency ([#53])

### Fixed

- PDF cache cannot be generated with a large `max_width` value ([#28])
- `show_hidden` option not working ([#47])
- Wrong task name when `shell` command has no arguments

### Improved

- Make code highlighting discardable ([#20])
- Improved performance of highlighting large JSON files ([#23])
- Wrap `stdout` with `BufWriter` to improve image preview performance ([#55])
- Improved bulk rename performance ([#54])

## [v0.1.2]

### Added

- New `sort` command to change sorting method on the fly ([#7])
- Which-key component to support multi-key chords ([#4])
- Hover the cursor over newly created files automatically ([#10])
- Make folders openable ([#9])
- Several default goto key mappings
- Support Überzug++ as the image preview backend for X11/Wayland ([#12])
- Cut input content to the system clipboard ([#6])
- Input component supports `undo` for cursor position
- Support for bracketed paste ([#5])

### Improved

- Cache directory size to avoid redundant calculations ([#11])

## [v0.1.1]

### Added

- Arrow keys are now bound for navigation by default (along with existing Vim-style bindings)
- Horizontal scrolling support for the `input` component
- Visual mode for the input component
- New `yank` and `paste` commands for the input component
- New `undo` and `redo` commands for the `input` component

### Fixed

- Cannot delete the last character of the input if at the end of the word

### Improved

- Decode images in a dedicated blocking thread to avoid blocking the UI

## [v0.1.0]

### Added

- Preset configurations
- New `open` command
- Select component for interactive `open`
- Plain text and archive preview
- Search files with `fd` and `rg`
- Jump around with `fzf` and `zoxide`
- Flat view for search results
- Precache images and videos
- Return to its parents if the CWD no longer exists
- Confirm when deleting files or exiting
- Custom status bar colors

### Fixed

- Build errors on Linux
- Number of remaining tasks cannot be updated

<!-- Link definitions -->

[Unreleased]: https://github.com/sxyazi/yazi/compare/shipped...HEAD
[v0.1.0]: https://github.com/sxyazi/yazi/releases/tag/v0.1.0
[v0.1.1]: https://github.com/sxyazi/yazi/compare/v0.1.0...v0.1.1
[v0.1.2]: https://github.com/sxyazi/yazi/compare/v0.1.1...v0.1.2
[v0.1.3]: https://github.com/sxyazi/yazi/compare/v0.1.2...v0.1.3
[v0.1.4]: https://github.com/sxyazi/yazi/compare/v0.1.3...v0.1.4
[v0.1.5]: https://github.com/sxyazi/yazi/compare/v0.1.4...v0.1.5
[v0.2.0]: https://github.com/sxyazi/yazi/compare/v0.1.5...v0.2.0
[v0.2.1]: https://github.com/sxyazi/yazi/compare/v0.2.0...v0.2.1
[v0.2.2]: https://github.com/sxyazi/yazi/compare/v0.2.1...v0.2.2
[v0.2.3]: https://github.com/sxyazi/yazi/compare/v0.2.2...v0.2.3
[v0.2.4]: https://github.com/sxyazi/yazi/compare/v0.2.3...v0.2.4
[v0.2.5]: https://github.com/sxyazi/yazi/compare/v0.2.4...v0.2.5
[v0.3.0]: https://github.com/sxyazi/yazi/compare/v0.2.5...v0.3.0
[v0.3.1]: https://github.com/sxyazi/yazi/compare/v0.3.0...v0.3.1
[v0.3.2]: https://github.com/sxyazi/yazi/compare/v0.3.1...v0.3.2
[v0.3.3]: https://github.com/sxyazi/yazi/compare/v0.3.2...v0.3.3
[v0.4.0]: https://github.com/sxyazi/yazi/compare/v0.3.3...v0.4.0
[v0.4.1]: https://github.com/sxyazi/yazi/compare/v0.4.0...v0.4.1
[v0.4.2]: https://github.com/sxyazi/yazi/compare/v0.4.1...v0.4.2
[v25.2.7]: https://github.com/sxyazi/yazi/compare/v0.4.2...v25.2.7
[v25.2.11]: https://github.com/sxyazi/yazi/compare/v25.2.7...v25.2.11
[v25.2.26]: https://github.com/sxyazi/yazi/compare/v25.2.11...v25.2.26
[v25.3.2]: https://github.com/sxyazi/yazi/compare/v25.2.26...v25.3.2
[v25.4.8]: https://github.com/sxyazi/yazi/compare/v25.3.2...v25.4.8
[v25.5.28]: https://github.com/sxyazi/yazi/compare/v25.4.8...v25.5.28
[v25.5.31]: https://github.com/sxyazi/yazi/compare/v25.5.28...v25.5.31
[v25.12.29]: https://github.com/sxyazi/yazi/compare/v25.5.31...v25.12.29
[v26.1.4]: https://github.com/sxyazi/yazi/compare/v25.12.29...v26.1.4
[#4]: https://github.com/sxyazi/yazi/pull/4
[#5]: https://github.com/sxyazi/yazi/pull/5
[#6]: https://github.com/sxyazi/yazi/pull/6
[#7]: https://github.com/sxyazi/yazi/pull/7
[#9]: https://github.com/sxyazi/yazi/pull/9
[#10]: https://github.com/sxyazi/yazi/pull/10
[#11]: https://github.com/sxyazi/yazi/pull/11
[#12]: https://github.com/sxyazi/yazi/pull/12
[#14]: https://github.com/sxyazi/yazi/pull/14
[#17]: https://github.com/sxyazi/yazi/pull/17
[#18]: https://github.com/sxyazi/yazi/pull/18
[#20]: https://github.com/sxyazi/yazi/pull/20
[#22]: https://github.com/sxyazi/yazi/pull/22
[#23]: https://github.com/sxyazi/yazi/pull/23
[#24]: https://github.com/sxyazi/yazi/pull/24
[#28]: https://github.com/sxyazi/yazi/pull/28
[#30]: https://github.com/sxyazi/yazi/pull/30
[#31]: https://github.com/sxyazi/yazi/pull/31
[#40]: https://github.com/sxyazi/yazi/pull/40
[#41]: https://github.com/sxyazi/yazi/pull/41
[#43]: https://github.com/sxyazi/yazi/pull/43
[#47]: https://github.com/sxyazi/yazi/pull/47
[#48]: https://github.com/sxyazi/yazi/pull/48
[#49]: https://github.com/sxyazi/yazi/pull/49
[#50]: https://github.com/sxyazi/yazi/pull/50
[#53]: https://github.com/sxyazi/yazi/pull/53
[#54]: https://github.com/sxyazi/yazi/pull/54
[#55]: https://github.com/sxyazi/yazi/pull/55
[#65]: https://github.com/sxyazi/yazi/pull/65
[#67]: https://github.com/sxyazi/yazi/pull/67
[#69]: https://github.com/sxyazi/yazi/pull/69
[#72]: https://github.com/sxyazi/yazi/pull/72
[#73]: https://github.com/sxyazi/yazi/pull/73
[#74]: https://github.com/sxyazi/yazi/pull/74
[#75]: https://github.com/sxyazi/yazi/pull/75
[#76]: https://github.com/sxyazi/yazi/pull/76
[#77]: https://github.com/sxyazi/yazi/pull/77
[#78]: https://github.com/sxyazi/yazi/pull/78
[#82]: https://github.com/sxyazi/yazi/pull/82
[#86]: https://github.com/sxyazi/yazi/pull/86
[#91]: https://github.com/sxyazi/yazi/pull/91
[#93]: https://github.com/sxyazi/yazi/pull/93
[#96]: https://github.com/sxyazi/yazi/pull/96
[#97]: https://github.com/sxyazi/yazi/pull/97
[#99]: https://github.com/sxyazi/yazi/pull/99
[#100]: https://github.com/sxyazi/yazi/pull/100
[#104]: https://github.com/sxyazi/yazi/pull/104
[#117]: https://github.com/sxyazi/yazi/pull/117
[#120]: https://github.com/sxyazi/yazi/pull/120
[#121]: https://github.com/sxyazi/yazi/pull/121
[#124]: https://github.com/sxyazi/yazi/pull/124
[#127]: https://github.com/sxyazi/yazi/pull/127
[#128]: https://github.com/sxyazi/yazi/pull/128
[#131]: https://github.com/sxyazi/yazi/pull/131
[#147]: https://github.com/sxyazi/yazi/pull/147
[#151]: https://github.com/sxyazi/yazi/pull/151
[#152]: https://github.com/sxyazi/yazi/pull/152
[#154]: https://github.com/sxyazi/yazi/pull/154
[#155]: https://github.com/sxyazi/yazi/pull/155
[#156]: https://github.com/sxyazi/yazi/pull/156
[#160]: https://github.com/sxyazi/yazi/pull/160
[#161]: https://github.com/sxyazi/yazi/pull/161
[#167]: https://github.com/sxyazi/yazi/pull/167
[#169]: https://github.com/sxyazi/yazi/pull/169
[#174]: https://github.com/sxyazi/yazi/pull/174
[#178]: https://github.com/sxyazi/yazi/pull/178
[#179]: https://github.com/sxyazi/yazi/pull/179
[#205]: https://github.com/sxyazi/yazi/pull/205
[#208]: https://github.com/sxyazi/yazi/pull/208
[#209]: https://github.com/sxyazi/yazi/pull/209
[#211]: https://github.com/sxyazi/yazi/pull/211
[#213]: https://github.com/sxyazi/yazi/pull/213
[#216]: https://github.com/sxyazi/yazi/pull/216
[#240]: https://github.com/sxyazi/yazi/pull/240
[#241]: https://github.com/sxyazi/yazi/pull/241
[#245]: https://github.com/sxyazi/yazi/pull/245
[#250]: https://github.com/sxyazi/yazi/pull/250
[#251]: https://github.com/sxyazi/yazi/pull/251
[#278]: https://github.com/sxyazi/yazi/pull/278
[#289]: https://github.com/sxyazi/yazi/pull/289
[#290]: https://github.com/sxyazi/yazi/pull/290
[#291]: https://github.com/sxyazi/yazi/pull/291
[#309]: https://github.com/sxyazi/yazi/pull/309
[#312]: https://github.com/sxyazi/yazi/pull/312
[#313]: https://github.com/sxyazi/yazi/pull/313
[#315]: https://github.com/sxyazi/yazi/pull/315
[#320]: https://github.com/sxyazi/yazi/pull/320
[#324]: https://github.com/sxyazi/yazi/pull/324
[#329]: https://github.com/sxyazi/yazi/pull/329
[#341]: https://github.com/sxyazi/yazi/pull/341
[#342]: https://github.com/sxyazi/yazi/pull/342
[#345]: https://github.com/sxyazi/yazi/pull/345
[#349]: https://github.com/sxyazi/yazi/pull/349
[#352]: https://github.com/sxyazi/yazi/pull/352
[#353]: https://github.com/sxyazi/yazi/pull/353
[#357]: https://github.com/sxyazi/yazi/pull/357
[#358]: https://github.com/sxyazi/yazi/pull/358
[#360]: https://github.com/sxyazi/yazi/pull/360
[#361]: https://github.com/sxyazi/yazi/pull/361
[#365]: https://github.com/sxyazi/yazi/pull/365
[#369]: https://github.com/sxyazi/yazi/pull/369
[#375]: https://github.com/sxyazi/yazi/pull/375
[#376]: https://github.com/sxyazi/yazi/pull/376
[#382]: https://github.com/sxyazi/yazi/pull/382
[#386]: https://github.com/sxyazi/yazi/pull/386
[#401]: https://github.com/sxyazi/yazi/pull/401
[#405]: https://github.com/sxyazi/yazi/pull/405
[#430]: https://github.com/sxyazi/yazi/pull/430
[#436]: https://github.com/sxyazi/yazi/pull/436
[#447]: https://github.com/sxyazi/yazi/pull/447
[#454]: https://github.com/sxyazi/yazi/pull/454
[#462]: https://github.com/sxyazi/yazi/pull/462
[#467]: https://github.com/sxyazi/yazi/pull/467
[#468]: https://github.com/sxyazi/yazi/pull/468
[#469]: https://github.com/sxyazi/yazi/pull/469
[#488]: https://github.com/sxyazi/yazi/pull/488
[#503]: https://github.com/sxyazi/yazi/pull/503
[#509]: https://github.com/sxyazi/yazi/pull/509
[#510]: https://github.com/sxyazi/yazi/pull/510
[#513]: https://github.com/sxyazi/yazi/pull/513
[#514]: https://github.com/sxyazi/yazi/pull/514
[#519]: https://github.com/sxyazi/yazi/pull/519
[#529]: https://github.com/sxyazi/yazi/pull/529
[#531]: https://github.com/sxyazi/yazi/pull/531
[#534]: https://github.com/sxyazi/yazi/pull/534
[#543]: https://github.com/sxyazi/yazi/pull/543
[#546]: https://github.com/sxyazi/yazi/pull/546
[#550]: https://github.com/sxyazi/yazi/pull/550
[#556]: https://github.com/sxyazi/yazi/pull/556
[#558]: https://github.com/sxyazi/yazi/pull/558
[#561]: https://github.com/sxyazi/yazi/pull/561
[#565]: https://github.com/sxyazi/yazi/pull/565
[#569]: https://github.com/sxyazi/yazi/pull/569
[#571]: https://github.com/sxyazi/yazi/pull/571
[#575]: https://github.com/sxyazi/yazi/pull/575
[#576]: https://github.com/sxyazi/yazi/pull/576
[#581]: https://github.com/sxyazi/yazi/pull/581
[#585]: https://github.com/sxyazi/yazi/pull/585
[#586]: https://github.com/sxyazi/yazi/pull/586
[#587]: https://github.com/sxyazi/yazi/pull/587
[#590]: https://github.com/sxyazi/yazi/pull/590
[#599]: https://github.com/sxyazi/yazi/pull/599
[#600]: https://github.com/sxyazi/yazi/pull/600
[#607]: https://github.com/sxyazi/yazi/pull/607
[#617]: https://github.com/sxyazi/yazi/pull/617
[#628]: https://github.com/sxyazi/yazi/pull/628
[#632]: https://github.com/sxyazi/yazi/pull/632
[#643]: https://github.com/sxyazi/yazi/pull/643
[#644]: https://github.com/sxyazi/yazi/pull/644
[#646]: https://github.com/sxyazi/yazi/pull/646
[#649]: https://github.com/sxyazi/yazi/pull/649
[#659]: https://github.com/sxyazi/yazi/pull/659
[#662]: https://github.com/sxyazi/yazi/pull/662
[#665]: https://github.com/sxyazi/yazi/pull/665
[#670]: https://github.com/sxyazi/yazi/pull/670
[#674]: https://github.com/sxyazi/yazi/pull/674
[#679]: https://github.com/sxyazi/yazi/pull/679
[#683]: https://github.com/sxyazi/yazi/pull/683
[#687]: https://github.com/sxyazi/yazi/pull/687
[#689]: https://github.com/sxyazi/yazi/pull/689
[#693]: https://github.com/sxyazi/yazi/pull/693
[#710]: https://github.com/sxyazi/yazi/pull/710
[#721]: https://github.com/sxyazi/yazi/pull/721
[#738]: https://github.com/sxyazi/yazi/pull/738
[#749]: https://github.com/sxyazi/yazi/pull/749
[#751]: https://github.com/sxyazi/yazi/pull/751
[#752]: https://github.com/sxyazi/yazi/pull/752
[#753]: https://github.com/sxyazi/yazi/pull/753
[#754]: https://github.com/sxyazi/yazi/pull/754
[#759]: https://github.com/sxyazi/yazi/pull/759
[#762]: https://github.com/sxyazi/yazi/pull/762
[#763]: https://github.com/sxyazi/yazi/pull/763
[#775]: https://github.com/sxyazi/yazi/pull/775
[#779]: https://github.com/sxyazi/yazi/pull/779
[#780]: https://github.com/sxyazi/yazi/pull/780
[#786]: https://github.com/sxyazi/yazi/pull/786
[#788]: https://github.com/sxyazi/yazi/pull/788
[#792]: https://github.com/sxyazi/yazi/pull/792
[#794]: https://github.com/sxyazi/yazi/pull/794
[#799]: https://github.com/sxyazi/yazi/pull/799
[#812]: https://github.com/sxyazi/yazi/pull/812
[#824]: https://github.com/sxyazi/yazi/pull/824
[#826]: https://github.com/sxyazi/yazi/pull/826
[#835]: https://github.com/sxyazi/yazi/pull/835
[#837]: https://github.com/sxyazi/yazi/pull/837
[#843]: https://github.com/sxyazi/yazi/pull/843
[#846]: https://github.com/sxyazi/yazi/pull/846
[#849]: https://github.com/sxyazi/yazi/pull/849
[#853]: https://github.com/sxyazi/yazi/pull/853
[#855]: https://github.com/sxyazi/yazi/pull/855
[#861]: https://github.com/sxyazi/yazi/pull/861
[#867]: https://github.com/sxyazi/yazi/pull/867
[#868]: https://github.com/sxyazi/yazi/pull/868
[#871]: https://github.com/sxyazi/yazi/pull/871
[#877]: https://github.com/sxyazi/yazi/pull/877
[#879]: https://github.com/sxyazi/yazi/pull/879
[#880]: https://github.com/sxyazi/yazi/pull/880
[#881]: https://github.com/sxyazi/yazi/pull/881
[#884]: https://github.com/sxyazi/yazi/pull/884
[#887]: https://github.com/sxyazi/yazi/pull/887
[#895]: https://github.com/sxyazi/yazi/pull/895
[#900]: https://github.com/sxyazi/yazi/pull/900
[#908]: https://github.com/sxyazi/yazi/pull/908
[#909]: https://github.com/sxyazi/yazi/pull/909
[#910]: https://github.com/sxyazi/yazi/pull/910
[#913]: https://github.com/sxyazi/yazi/pull/913
[#917]: https://github.com/sxyazi/yazi/pull/917
[#920]: https://github.com/sxyazi/yazi/pull/920
[#925]: https://github.com/sxyazi/yazi/pull/925
[#926]: https://github.com/sxyazi/yazi/pull/926
[#928]: https://github.com/sxyazi/yazi/pull/928
[#931]: https://github.com/sxyazi/yazi/pull/931
[#933]: https://github.com/sxyazi/yazi/pull/933
[#937]: https://github.com/sxyazi/yazi/pull/937
[#940]: https://github.com/sxyazi/yazi/pull/940
[#944]: https://github.com/sxyazi/yazi/pull/944
[#948]: https://github.com/sxyazi/yazi/pull/948
[#958]: https://github.com/sxyazi/yazi/pull/958
[#975]: https://github.com/sxyazi/yazi/pull/975
[#977]: https://github.com/sxyazi/yazi/pull/977
[#980]: https://github.com/sxyazi/yazi/pull/980
[#985]: https://github.com/sxyazi/yazi/pull/985
[#997]: https://github.com/sxyazi/yazi/pull/997
[#1003]: https://github.com/sxyazi/yazi/pull/1003
[#1004]: https://github.com/sxyazi/yazi/pull/1004
[#1005]: https://github.com/sxyazi/yazi/pull/1005
[#1025]: https://github.com/sxyazi/yazi/pull/1025
[#1033]: https://github.com/sxyazi/yazi/pull/1033
[#1038]: https://github.com/sxyazi/yazi/pull/1038
[#1048]: https://github.com/sxyazi/yazi/pull/1048
[#1050]: https://github.com/sxyazi/yazi/pull/1050
[#1053]: https://github.com/sxyazi/yazi/pull/1053
[#1069]: https://github.com/sxyazi/yazi/pull/1069
[#1070]: https://github.com/sxyazi/yazi/pull/1070
[#1081]: https://github.com/sxyazi/yazi/pull/1081
[#1082]: https://github.com/sxyazi/yazi/pull/1082
[#1086]: https://github.com/sxyazi/yazi/pull/1086
[#1094]: https://github.com/sxyazi/yazi/pull/1094
[#1110]: https://github.com/sxyazi/yazi/pull/1110
[#1111]: https://github.com/sxyazi/yazi/pull/1111
[#1139]: https://github.com/sxyazi/yazi/pull/1139
[#1151]: https://github.com/sxyazi/yazi/pull/1151
[#1159]: https://github.com/sxyazi/yazi/pull/1159
[#1167]: https://github.com/sxyazi/yazi/pull/1167
[#1169]: https://github.com/sxyazi/yazi/pull/1169
[#1185]: https://github.com/sxyazi/yazi/pull/1185
[#1220]: https://github.com/sxyazi/yazi/pull/1220
[#1227]: https://github.com/sxyazi/yazi/pull/1227
[#1232]: https://github.com/sxyazi/yazi/pull/1232
[#1238]: https://github.com/sxyazi/yazi/pull/1238
[#1241]: https://github.com/sxyazi/yazi/pull/1241
[#1249]: https://github.com/sxyazi/yazi/pull/1249
[#1268]: https://github.com/sxyazi/yazi/pull/1268
[#1270]: https://github.com/sxyazi/yazi/pull/1270
[#1291]: https://github.com/sxyazi/yazi/pull/1291
[#1295]: https://github.com/sxyazi/yazi/pull/1295
[#1305]: https://github.com/sxyazi/yazi/pull/1305
[#1321]: https://github.com/sxyazi/yazi/pull/1321
[#1361]: https://github.com/sxyazi/yazi/pull/1361
[#1395]: https://github.com/sxyazi/yazi/pull/1395
[#1412]: https://github.com/sxyazi/yazi/pull/1412
[#1422]: https://github.com/sxyazi/yazi/pull/1422
[#1428]: https://github.com/sxyazi/yazi/pull/1428
[#1431]: https://github.com/sxyazi/yazi/pull/1431
[#1434]: https://github.com/sxyazi/yazi/pull/1434
[#1439]: https://github.com/sxyazi/yazi/pull/1439
[#1443]: https://github.com/sxyazi/yazi/pull/1443
[#1446]: https://github.com/sxyazi/yazi/pull/1446
[#1448]: https://github.com/sxyazi/yazi/pull/1448
[#1451]: https://github.com/sxyazi/yazi/pull/1451
[#1461]: https://github.com/sxyazi/yazi/pull/1461
[#1464]: https://github.com/sxyazi/yazi/pull/1464
[#1467]: https://github.com/sxyazi/yazi/pull/1467
[#1468]: https://github.com/sxyazi/yazi/pull/1468
[#1473]: https://github.com/sxyazi/yazi/pull/1473
[#1474]: https://github.com/sxyazi/yazi/pull/1474
[#1482]: https://github.com/sxyazi/yazi/pull/1482
[#1497]: https://github.com/sxyazi/yazi/pull/1497
[#1500]: https://github.com/sxyazi/yazi/pull/1500
[#1505]: https://github.com/sxyazi/yazi/pull/1505
[#1512]: https://github.com/sxyazi/yazi/pull/1512
[#1528]: https://github.com/sxyazi/yazi/pull/1528
[#1541]: https://github.com/sxyazi/yazi/pull/1541
[#1542]: https://github.com/sxyazi/yazi/pull/1542
[#1550]: https://github.com/sxyazi/yazi/pull/1550
[#1551]: https://github.com/sxyazi/yazi/pull/1551
[#1556]: https://github.com/sxyazi/yazi/pull/1556
[#1562]: https://github.com/sxyazi/yazi/pull/1562
[#1566]: https://github.com/sxyazi/yazi/pull/1566
[#1568]: https://github.com/sxyazi/yazi/pull/1568
[#1574]: https://github.com/sxyazi/yazi/pull/1574
[#1583]: https://github.com/sxyazi/yazi/pull/1583
[#1588]: https://github.com/sxyazi/yazi/pull/1588
[#1590]: https://github.com/sxyazi/yazi/pull/1590
[#1591]: https://github.com/sxyazi/yazi/pull/1591
[#1605]: https://github.com/sxyazi/yazi/pull/1605
[#1614]: https://github.com/sxyazi/yazi/pull/1614
[#1622]: https://github.com/sxyazi/yazi/pull/1622
[#1639]: https://github.com/sxyazi/yazi/pull/1639
[#1648]: https://github.com/sxyazi/yazi/pull/1648
[#1650]: https://github.com/sxyazi/yazi/pull/1650
[#1652]: https://github.com/sxyazi/yazi/pull/1652
[#1666]: https://github.com/sxyazi/yazi/pull/1666
[#1667]: https://github.com/sxyazi/yazi/pull/1667
[#1680]: https://github.com/sxyazi/yazi/pull/1680
[#1682]: https://github.com/sxyazi/yazi/pull/1682
[#1689]: https://github.com/sxyazi/yazi/pull/1689
[#1695]: https://github.com/sxyazi/yazi/pull/1695
[#1704]: https://github.com/sxyazi/yazi/pull/1704
[#1737]: https://github.com/sxyazi/yazi/pull/1737
[#1745]: https://github.com/sxyazi/yazi/pull/1745
[#1761]: https://github.com/sxyazi/yazi/pull/1761
[#1762]: https://github.com/sxyazi/yazi/pull/1762
[#1772]: https://github.com/sxyazi/yazi/pull/1772
[#1773]: https://github.com/sxyazi/yazi/pull/1773
[#1776]: https://github.com/sxyazi/yazi/pull/1776
[#1782]: https://github.com/sxyazi/yazi/pull/1782
[#1784]: https://github.com/sxyazi/yazi/pull/1784
[#1789]: https://github.com/sxyazi/yazi/pull/1789
[#1792]: https://github.com/sxyazi/yazi/pull/1792
[#1801]: https://github.com/sxyazi/yazi/pull/1801
[#1802]: https://github.com/sxyazi/yazi/pull/1802
[#1807]: https://github.com/sxyazi/yazi/pull/1807
[#1808]: https://github.com/sxyazi/yazi/pull/1808
[#1816]: https://github.com/sxyazi/yazi/pull/1816
[#1832]: https://github.com/sxyazi/yazi/pull/1832
[#1833]: https://github.com/sxyazi/yazi/pull/1833
[#1846]: https://github.com/sxyazi/yazi/pull/1846
[#1849]: https://github.com/sxyazi/yazi/pull/1849
[#1863]: https://github.com/sxyazi/yazi/pull/1863
[#1877]: https://github.com/sxyazi/yazi/pull/1877
[#1882]: https://github.com/sxyazi/yazi/pull/1882
[#1884]: https://github.com/sxyazi/yazi/pull/1884
[#1891]: https://github.com/sxyazi/yazi/pull/1891
[#1903]: https://github.com/sxyazi/yazi/pull/1903
[#1926]: https://github.com/sxyazi/yazi/pull/1926
[#1927]: https://github.com/sxyazi/yazi/pull/1927
[#1928]: https://github.com/sxyazi/yazi/pull/1928
[#1939]: https://github.com/sxyazi/yazi/pull/1939
[#1945]: https://github.com/sxyazi/yazi/pull/1945
[#1946]: https://github.com/sxyazi/yazi/pull/1946
[#1953]: https://github.com/sxyazi/yazi/pull/1953
[#1962]: https://github.com/sxyazi/yazi/pull/1962
[#1966]: https://github.com/sxyazi/yazi/pull/1966
[#1973]: https://github.com/sxyazi/yazi/pull/1973
[#1979]: https://github.com/sxyazi/yazi/pull/1979
[#1980]: https://github.com/sxyazi/yazi/pull/1980
[#1982]: https://github.com/sxyazi/yazi/pull/1982
[#1984]: https://github.com/sxyazi/yazi/pull/1984
[#1995]: https://github.com/sxyazi/yazi/pull/1995
[#2002]: https://github.com/sxyazi/yazi/pull/2002
[#2003]: https://github.com/sxyazi/yazi/pull/2003
[#2004]: https://github.com/sxyazi/yazi/pull/2004
[#2006]: https://github.com/sxyazi/yazi/pull/2006
[#2014]: https://github.com/sxyazi/yazi/pull/2014
[#2017]: https://github.com/sxyazi/yazi/pull/2017
[#2020]: https://github.com/sxyazi/yazi/pull/2020
[#2025]: https://github.com/sxyazi/yazi/pull/2025
[#2030]: https://github.com/sxyazi/yazi/pull/2030
[#2041]: https://github.com/sxyazi/yazi/pull/2041
[#2043]: https://github.com/sxyazi/yazi/pull/2043
[#2052]: https://github.com/sxyazi/yazi/pull/2052
[#2058]: https://github.com/sxyazi/yazi/pull/2058
[#2060]: https://github.com/sxyazi/yazi/pull/2060
[#2064]: https://github.com/sxyazi/yazi/pull/2064
[#2068]: https://github.com/sxyazi/yazi/pull/2068
[#2071]: https://github.com/sxyazi/yazi/pull/2071
[#2072]: https://github.com/sxyazi/yazi/pull/2072
[#2077]: https://github.com/sxyazi/yazi/pull/2077
[#2093]: https://github.com/sxyazi/yazi/pull/2093
[#2095]: https://github.com/sxyazi/yazi/pull/2095
[#2105]: https://github.com/sxyazi/yazi/pull/2105
[#2110]: https://github.com/sxyazi/yazi/pull/2110
[#2122]: https://github.com/sxyazi/yazi/pull/2122
[#2132]: https://github.com/sxyazi/yazi/pull/2132
[#2143]: https://github.com/sxyazi/yazi/pull/2143
[#2149]: https://github.com/sxyazi/yazi/pull/2149
[#2168]: https://github.com/sxyazi/yazi/pull/2168
[#2173]: https://github.com/sxyazi/yazi/pull/2173
[#2181]: https://github.com/sxyazi/yazi/pull/2181
[#2185]: https://github.com/sxyazi/yazi/pull/2185
[#2186]: https://github.com/sxyazi/yazi/pull/2186
[#2188]: https://github.com/sxyazi/yazi/pull/2188
[#2199]: https://github.com/sxyazi/yazi/pull/2199
[#2205]: https://github.com/sxyazi/yazi/pull/2205
[#2210]: https://github.com/sxyazi/yazi/pull/2210
[#2224]: https://github.com/sxyazi/yazi/pull/2224
[#2233]: https://github.com/sxyazi/yazi/pull/2233
[#2234]: https://github.com/sxyazi/yazi/pull/2234
[#2242]: https://github.com/sxyazi/yazi/pull/2242
[#2245]: https://github.com/sxyazi/yazi/pull/2245
[#2247]: https://github.com/sxyazi/yazi/pull/2247
[#2253]: https://github.com/sxyazi/yazi/pull/2253
[#2257]: https://github.com/sxyazi/yazi/pull/2257
[#2290]: https://github.com/sxyazi/yazi/pull/2290
[#2294]: https://github.com/sxyazi/yazi/pull/2294
[#2298]: https://github.com/sxyazi/yazi/pull/2298
[#2299]: https://github.com/sxyazi/yazi/pull/2299
[#2310]: https://github.com/sxyazi/yazi/pull/2310
[#2313]: https://github.com/sxyazi/yazi/pull/2313
[#2314]: https://github.com/sxyazi/yazi/pull/2314
[#2319]: https://github.com/sxyazi/yazi/pull/2319
[#2321]: https://github.com/sxyazi/yazi/pull/2321
[#2326]: https://github.com/sxyazi/yazi/pull/2326
[#2327]: https://github.com/sxyazi/yazi/pull/2327
[#2331]: https://github.com/sxyazi/yazi/pull/2331
[#2337]: https://github.com/sxyazi/yazi/pull/2337
[#2343]: https://github.com/sxyazi/yazi/pull/2343
[#2355]: https://github.com/sxyazi/yazi/pull/2355
[#2366]: https://github.com/sxyazi/yazi/pull/2366
[#2383]: https://github.com/sxyazi/yazi/pull/2383
[#2389]: https://github.com/sxyazi/yazi/pull/2389
[#2391]: https://github.com/sxyazi/yazi/pull/2391
[#2392]: https://github.com/sxyazi/yazi/pull/2392
[#2393]: https://github.com/sxyazi/yazi/pull/2393
[#2397]: https://github.com/sxyazi/yazi/pull/2397
[#2399]: https://github.com/sxyazi/yazi/pull/2399
[#2403]: https://github.com/sxyazi/yazi/pull/2403
[#2405]: https://github.com/sxyazi/yazi/pull/2405
[#2413]: https://github.com/sxyazi/yazi/pull/2413
[#2418]: https://github.com/sxyazi/yazi/pull/2418
[#2425]: https://github.com/sxyazi/yazi/pull/2425
[#2427]: https://github.com/sxyazi/yazi/pull/2427
[#2431]: https://github.com/sxyazi/yazi/pull/2431
[#2439]: https://github.com/sxyazi/yazi/pull/2439
[#2442]: https://github.com/sxyazi/yazi/pull/2442
[#2444]: https://github.com/sxyazi/yazi/pull/2444
[#2449]: https://github.com/sxyazi/yazi/pull/2449
[#2452]: https://github.com/sxyazi/yazi/pull/2452
[#2456]: https://github.com/sxyazi/yazi/pull/2456
[#2458]: https://github.com/sxyazi/yazi/pull/2458
[#2461]: https://github.com/sxyazi/yazi/pull/2461
[#2471]: https://github.com/sxyazi/yazi/pull/2471
[#2476]: https://github.com/sxyazi/yazi/pull/2476
[#2485]: https://github.com/sxyazi/yazi/pull/2485
[#2487]: https://github.com/sxyazi/yazi/pull/2487
[#2490]: https://github.com/sxyazi/yazi/pull/2490
[#2492]: https://github.com/sxyazi/yazi/pull/2492
[#2494]: https://github.com/sxyazi/yazi/pull/2494
[#2503]: https://github.com/sxyazi/yazi/pull/2503
[#2508]: https://github.com/sxyazi/yazi/pull/2508
[#2522]: https://github.com/sxyazi/yazi/pull/2522
[#2526]: https://github.com/sxyazi/yazi/pull/2526
[#2527]: https://github.com/sxyazi/yazi/pull/2527
[#2530]: https://github.com/sxyazi/yazi/pull/2530
[#2533]: https://github.com/sxyazi/yazi/pull/2533
[#2540]: https://github.com/sxyazi/yazi/pull/2540
[#2543]: https://github.com/sxyazi/yazi/pull/2543
[#2546]: https://github.com/sxyazi/yazi/pull/2546
[#2553]: https://github.com/sxyazi/yazi/pull/2553
[#2560]: https://github.com/sxyazi/yazi/pull/2560
[#2572]: https://github.com/sxyazi/yazi/pull/2572
[#2574]: https://github.com/sxyazi/yazi/pull/2574
[#2578]: https://github.com/sxyazi/yazi/pull/2578
[#2581]: https://github.com/sxyazi/yazi/pull/2581
[#2589]: https://github.com/sxyazi/yazi/pull/2589
[#2594]: https://github.com/sxyazi/yazi/pull/2594
[#2602]: https://github.com/sxyazi/yazi/pull/2602
[#2609]: https://github.com/sxyazi/yazi/pull/2609
[#2636]: https://github.com/sxyazi/yazi/pull/2636
[#2640]: https://github.com/sxyazi/yazi/pull/2640
[#2653]: https://github.com/sxyazi/yazi/pull/2653
[#2657]: https://github.com/sxyazi/yazi/pull/2657
[#2664]: https://github.com/sxyazi/yazi/pull/2664
[#2675]: https://github.com/sxyazi/yazi/pull/2675
[#2678]: https://github.com/sxyazi/yazi/pull/2678
[#2683]: https://github.com/sxyazi/yazi/pull/2683
[#2691]: https://github.com/sxyazi/yazi/pull/2691
[#2695]: https://github.com/sxyazi/yazi/pull/2695
[#2696]: https://github.com/sxyazi/yazi/pull/2696
[#2697]: https://github.com/sxyazi/yazi/pull/2697
[#2700]: https://github.com/sxyazi/yazi/pull/2700
[#2706]: https://github.com/sxyazi/yazi/pull/2706
[#2707]: https://github.com/sxyazi/yazi/pull/2707
[#2709]: https://github.com/sxyazi/yazi/pull/2709
[#2723]: https://github.com/sxyazi/yazi/pull/2723
[#2734]: https://github.com/sxyazi/yazi/pull/2734
[#2743]: https://github.com/sxyazi/yazi/pull/2743
[#2745]: https://github.com/sxyazi/yazi/pull/2745
[#2752]: https://github.com/sxyazi/yazi/pull/2752
[#2753]: https://github.com/sxyazi/yazi/pull/2753
[#2754]: https://github.com/sxyazi/yazi/pull/2754
[#2759]: https://github.com/sxyazi/yazi/pull/2759
[#2764]: https://github.com/sxyazi/yazi/pull/2764
[#2765]: https://github.com/sxyazi/yazi/pull/2765
[#2769]: https://github.com/sxyazi/yazi/pull/2769
[#2770]: https://github.com/sxyazi/yazi/pull/2770
[#2778]: https://github.com/sxyazi/yazi/pull/2778
[#2802]: https://github.com/sxyazi/yazi/pull/2802
[#2803]: https://github.com/sxyazi/yazi/pull/2803
[#2807]: https://github.com/sxyazi/yazi/pull/2807
[#2810]: https://github.com/sxyazi/yazi/pull/2810
[#2811]: https://github.com/sxyazi/yazi/pull/2811
[#2814]: https://github.com/sxyazi/yazi/pull/2814
[#2820]: https://github.com/sxyazi/yazi/pull/2820
[#2834]: https://github.com/sxyazi/yazi/pull/2834
[#2841]: https://github.com/sxyazi/yazi/pull/2841
[#2843]: https://github.com/sxyazi/yazi/pull/2843
[#2849]: https://github.com/sxyazi/yazi/pull/2849
[#2855]: https://github.com/sxyazi/yazi/pull/2855
[#2861]: https://github.com/sxyazi/yazi/pull/2861
[#2862]: https://github.com/sxyazi/yazi/pull/2862
[#2864]: https://github.com/sxyazi/yazi/pull/2864
[#2875]: https://github.com/sxyazi/yazi/pull/2875
[#2879]: https://github.com/sxyazi/yazi/pull/2879
[#2880]: https://github.com/sxyazi/yazi/pull/2880
[#2884]: https://github.com/sxyazi/yazi/pull/2884
[#2889]: https://github.com/sxyazi/yazi/pull/2889
[#2890]: https://github.com/sxyazi/yazi/pull/2890
[#2895]: https://github.com/sxyazi/yazi/pull/2895
[#2904]: https://github.com/sxyazi/yazi/pull/2904
[#2906]: https://github.com/sxyazi/yazi/pull/2906
[#2914]: https://github.com/sxyazi/yazi/pull/2914
[#2915]: https://github.com/sxyazi/yazi/pull/2915
[#2917]: https://github.com/sxyazi/yazi/pull/2917
[#2921]: https://github.com/sxyazi/yazi/pull/2921
[#2925]: https://github.com/sxyazi/yazi/pull/2925
[#2927]: https://github.com/sxyazi/yazi/pull/2927
[#2931]: https://github.com/sxyazi/yazi/pull/2931
[#2932]: https://github.com/sxyazi/yazi/pull/2932
[#2935]: https://github.com/sxyazi/yazi/pull/2935
[#2939]: https://github.com/sxyazi/yazi/pull/2939
[#2941]: https://github.com/sxyazi/yazi/pull/2941
[#2958]: https://github.com/sxyazi/yazi/pull/2958
[#2959]: https://github.com/sxyazi/yazi/pull/2959
[#2964]: https://github.com/sxyazi/yazi/pull/2964
[#2984]: https://github.com/sxyazi/yazi/pull/2984
[#2997]: https://github.com/sxyazi/yazi/pull/2997
[#3005]: https://github.com/sxyazi/yazi/pull/3005
[#3008]: https://github.com/sxyazi/yazi/pull/3008
[#3023]: https://github.com/sxyazi/yazi/pull/3023
[#3034]: https://github.com/sxyazi/yazi/pull/3034
[#3035]: https://github.com/sxyazi/yazi/pull/3035
[#3037]: https://github.com/sxyazi/yazi/pull/3037
[#3038]: https://github.com/sxyazi/yazi/pull/3038
[#3059]: https://github.com/sxyazi/yazi/pull/3059
[#3067]: https://github.com/sxyazi/yazi/pull/3067
[#3083]: https://github.com/sxyazi/yazi/pull/3083
[#3084]: https://github.com/sxyazi/yazi/pull/3084
[#3091]: https://github.com/sxyazi/yazi/pull/3091
[#3094]: https://github.com/sxyazi/yazi/pull/3094
[#3108]: https://github.com/sxyazi/yazi/pull/3108
[#3117]: https://github.com/sxyazi/yazi/pull/3117
[#3121]: https://github.com/sxyazi/yazi/pull/3121
[#3128]: https://github.com/sxyazi/yazi/pull/3128
[#3131]: https://github.com/sxyazi/yazi/pull/3131
[#3134]: https://github.com/sxyazi/yazi/pull/3134
[#3141]: https://github.com/sxyazi/yazi/pull/3141
[#3154]: https://github.com/sxyazi/yazi/pull/3154
[#3166]: https://github.com/sxyazi/yazi/pull/3166
[#3169]: https://github.com/sxyazi/yazi/pull/3169
[#3170]: https://github.com/sxyazi/yazi/pull/3170
[#3172]: https://github.com/sxyazi/yazi/pull/3172
[#3187]: https://github.com/sxyazi/yazi/pull/3187
[#3189]: https://github.com/sxyazi/yazi/pull/3189
[#3190]: https://github.com/sxyazi/yazi/pull/3190
[#3198]: https://github.com/sxyazi/yazi/pull/3198
[#3200]: https://github.com/sxyazi/yazi/pull/3200
[#3201]: https://github.com/sxyazi/yazi/pull/3201
[#3203]: https://github.com/sxyazi/yazi/pull/3203
[#3209]: https://github.com/sxyazi/yazi/pull/3209
[#3222]: https://github.com/sxyazi/yazi/pull/3222
[#3225]: https://github.com/sxyazi/yazi/pull/3225
[#3226]: https://github.com/sxyazi/yazi/pull/3226
[#3232]: https://github.com/sxyazi/yazi/pull/3232
[#3235]: https://github.com/sxyazi/yazi/pull/3235
[#3243]: https://github.com/sxyazi/yazi/pull/3243
[#3250]: https://github.com/sxyazi/yazi/pull/3250
[#3264]: https://github.com/sxyazi/yazi/pull/3264
[#3268]: https://github.com/sxyazi/yazi/pull/3268
[#3271]: https://github.com/sxyazi/yazi/pull/3271
[#3286]: https://github.com/sxyazi/yazi/pull/3286
[#3290]: https://github.com/sxyazi/yazi/pull/3290
[#3313]: https://github.com/sxyazi/yazi/pull/3313
[#3317]: https://github.com/sxyazi/yazi/pull/3317
[#3358]: https://github.com/sxyazi/yazi/pull/3358
[#3360]: https://github.com/sxyazi/yazi/pull/3360
[#3361]: https://github.com/sxyazi/yazi/pull/3361
[#3364]: https://github.com/sxyazi/yazi/pull/3364
[#3369]: https://github.com/sxyazi/yazi/pull/3369
[#3383]: https://github.com/sxyazi/yazi/pull/3383
[#3385]: https://github.com/sxyazi/yazi/pull/3385
[#3387]: https://github.com/sxyazi/yazi/pull/3387
[#3391]: https://github.com/sxyazi/yazi/pull/3391
[#3393]: https://github.com/sxyazi/yazi/pull/3393
[#3396]: https://github.com/sxyazi/yazi/pull/3396
[#3419]: https://github.com/sxyazi/yazi/pull/3419
[#3422]: https://github.com/sxyazi/yazi/pull/3422
[#3429]: https://github.com/sxyazi/yazi/pull/3429
[#3456]: https://github.com/sxyazi/yazi/pull/3456
[#3467]: https://github.com/sxyazi/yazi/pull/3467
[#3477]: https://github.com/sxyazi/yazi/pull/3477
[#3482]: https://github.com/sxyazi/yazi/pull/3482
[#3494]: https://github.com/sxyazi/yazi/pull/3494
