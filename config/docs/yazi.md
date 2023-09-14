# Yazi

## manager

- layout: Manager layout by ratio, 3-element array

  - `[1, 4, 3]`: 1/8 width for parent, 4/8 width for current, 3/8 width for preview

- sort_by: File sorting method

  - `"alphabetical"`: Sort alphabetically, e.g. `1.md` < `10.md` < `2.md`
  - `"created"`: Sort by creation time
  - `"modified"`: Sort by last modified time
  - `"natural"`: Sort naturally, e.g. `1.md` < `2.md` < `10.md`
  - `"size"`: Sort by file size

- sort_sensitive: Sort case-sensitively

  - `true`: Case-sensitive
  - `false`: Case-insensitive

- sort_reverse: Display files in reverse order

  - `true`: Reverse order
  - `false`: Normal order

- sort_dir_first: Display directories first

  - `true`: Directories first
  - `false`: Respects `sort_by` and `sort_reverse` only

- show_hidden: Show hidden files

  - `true`: Show
  - `false`: Do not show

- show_symlink: Show the path of the symlink file point to, after the filename

  - `true`: Show
  - `false`: Do not show

## preview

- tab_size: Tab width
- max_width: Maximum preview width for images and videos
- max_height: Maximum preview height for images and videos
- cache_dir: The system cache directory is used by default, and the cached files will go away on a reboot automatically. If you want to make it more persistent, you can specify the cache directory manually as an absolute path.

## opener

Configure available openers, for example:

```toml
[opener]
archive = [
	{ exec = 'unar "$1"' },
]
text = [
	{ exec = 'nvim "$@"', block = true },
]
# ...
```

Available parameters are as follows:

- exec: The command to open the selected files, with the following variables available:
  - `$n`: The N-th selected file, starting from 1
  - `$@`: All selected files
  - `foo`: Literal string to be passed
- block: Open in a blocking manner. After setting this, Yazi will hide into a secondary screen and display the program on the main screen until it exits. During this time, it can receive I/O signals, which is useful for interactive programs.

## open

Set rules for opening specific files, for example:

```toml
[open]
rules = [
	{ mime = "text/*", use = "text" },
	{ mime = "image/*", use = "image" },

	# { mime = "application/json", use = "text" },
	{ name = "*.json", use = "text" },
]
```

Available rule parameters are as follows:

- name: Glob expression for matching the file name. Case insensitive by default, add `\s` to the beginning to make it sensitive.
- mime: Glob expression for matching the MIME type. Case insensitive by default, add `\s` to the beginning to make it sensitive.
- use: Opener name corresponding to the names in the opener section.

## tasks

- micro_workers: Maximum number of concurrent micro-tasks
- macro_workers: Maximum number of concurrent macro-tasks
- bizarre_retry: Maximum number of retries when a bizarre failure occurs
