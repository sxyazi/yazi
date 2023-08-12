# Yazi

## manager

- sort_by: File sorting method

  - `"alphabetical"`: Sort alphabetically
  - `"created"`: Sort by creation time
  - `"modified"`: Sort by last modified time
  - `"size"`: Sort by file size

- sort_reverse: Display files in reverse order

  - `true`: Reverse order
  - `false`: Normal order

- sort_dir_first: Display directories first

  - `true`: Directories first
  - `false`: Respects `sort_by` and `sort_reverse` only

- show_hidden: Show hidden files

  - `true`: Show
  - `false`: Do not show

## preview

- tab_size: Tab width
- max_width: Maximum preview width for images and videos
- max_height: Maximum preview height for images and videos

## opener

Configure available openers, for example:

```toml
[opener]
archive = [
	{ exec = "unar $1" },
]
text = [
	{ exec = "nvim $*", block = true },
]
# ...
```

Available parameters are as follows:

- exec: The command to open the selected files, with the following variables available:
  - `$n`: The N-th selected file
  - `$*`: All selected files
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

- name: Glob expression for matching the file name
- mime: Glob expression for matching the MIME type
- use: Opener name corresponding to the names in the opener section.
