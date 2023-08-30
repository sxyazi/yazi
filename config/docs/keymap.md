# Keymap

## manager

- escape: Exit visual mode, clear selected, or cancel search.
- quit: Exit the process.
- close: Close the current tab; if it is the last tab, then exit the process.

### Navigation

- arrow

  - `n`: Move the cursor up or down by n lines. Use negative values to move up and positive values to move down.

- leave: Go back to the parent directory.
- enter: Enter the child directory.
- back: Go back to the previous directory.
- forward: Go forward to the next directory.
- peek

  - `n`: Peek up or down at file contents in the preview. Use negative values to peek up and positive values to peek down.

- cd: Change the current directory.

  - `path`: the path to change to.
  - `--interactive`: Use an interactive UI to input the path.

### Selection

- select

  - `--state=true`: Select the current file.
  - `--state=false`: Deselect the current file.
  - `--state=none`: Default, toggle the selection state of the current file.

- select_all

  - `--state=true`: Select all files.
  - `--state=false`: Deselect all files.
  - `--state=none`: Default, toggle the selection state of all files.

- visual_mode: Enter visual mode (selection mode).

  - `--unset`: Enter visual mode (unset mode).

### Operation

- open: Open the selected files.

  - `--interactive`: Open the selected files with an interactive UI to choose the opening method.

- yank: Copy the selected files.

  - `--cut`: Cut the selected files.

- paste: Paste the files.

  - `--force`: Overwrite the destination file if it exists.
  - `--follow`: Copy the file pointed to by a symbolic link, rather than the link itself. Only valid during copying.

- remove: Move the files to the trash/recycle bin.

  - `--permanently`: Permanently delete the files.

- create: Create a file or directory (ends with `/` for directories).
- rename: Rename a file or directory.
- copy: Copy the path of files or directories that are selected or hovered on.

  - `path`: Copy the full absolute path.
  - `dirname`: Copy the path of the parent directory.
  - `filename`: Copy the name of the file.
  - `name_without_ext`: Copy the name of the file without the extension.

- shell: Run a shell command.

  - `exec`: Optional, command template to be run.
  - `--block`: Block the UI until the command finishes.
  - `--confirm`: When the template is provided, run it directly, no input UI was shown.

- hidden: Set the visibility of hidden files.

  - `show`: Show hidden files.
  - `hide`: Hide hidden files.
  - `toggle`: Default, toggle the hidden state.

- search

  - `rg`: Search files by content using ripgrep.
  - `fd`: Search files by name using fd.
  - `none`: Default, cancel the ongoing search.

- jump

  - `fzf`: Jump to a directory, or reveal a file using fzf.
  - `zoxide`: Jump to a directory using zoxide.

- sort

  - `by`
    - `"alphabetical"`: Sort alphabetically, e.g. `1.md` < `10.md` < `2.md`
    - `"created"`: Sort by creation time.
    - `"modified"`: Sort by last modified time.
    - `"natural"`: Sort naturally, e.g. `1.md` < `2.md` < `10.md`
    - `"size"`: Sort by file size.
  - `--reverse`: Display files in reverse order.
  - `--dir_first`: Display directories first.

### Tabs

- tab_create

  - `path`: Create a new tab using the specified path.
  - `--current`: Create a new tab using the current path.

- tab_close

  - `n`: Close the tab at position n, starting from 0.

- tab_switch

  - `n`: Switch to the tab at position n, starting from 0.
  - `--relative`: Switch to the tab at a position relative to the current tab. The value of n can be negative when using this parameter.

- tab_swap

  - `n`: Swap the current tab with the tab at position n, where negative values move the tab forward, and positive values move it backward.

### Tasks

- tasks_show: Show the task manager.

### Help

- help: Open the help menu.

## tasks

- close: Hide the task manager.
- arrow:
  - `-1`: Move the cursor up 1 line.
  - `1`: Move the cursor down 1 line.
- inspect: Inspect the task.
- cancel: Cancel the task.
- help: Open the help menu.

## select

- close: Cancel selection.
  - `--submit`: Submit the selection.
- arrow
  - `n`: Move the cursor up or down n lines. Negative value for up, positive value for down.
- help: Open the help menu.

## input

- close: Cancel input.

  - `--submit`: Submit the input.

- escape: Go back the normal mode, or cancel input.
- move: Move the cursor left or right.

  - `n`: Move the cursor n characters left or right. Negative value for left, positive value for right.
  - `--in-operating`: Move the cursor only if its currently waiting for an operation.

### Normal mode

- insert: Enter insert mode.

  - `--append`: Insert after the cursor.

- visual: Enter visual mode.
- backward: Move to the beginning of the previous word.
- forward: Move to the beginning of the next word.

  - `--end-of-word`: Move to the end of the next word.

- delete: Delete the selected characters.

  - `--cut`: Cut the selected characters into clipboard, instead of only deleting them.
  - `--insert`: Delete and enter insert mode.

- yank: Copy the selected characters.
- paste: Paste the copied characters after the cursor.

  - `--before`: Paste the copied characters before the cursor.

- undo: Undo the last operation.
- redo: Redo the last operation.

- help: Open the help menu.

### Insert mode

- close: Cancel input.

  - `--submit`: Submit the input.

- escape: Cancel insert mode and enter normal mode.
- backspace: Delete the character before the cursor.

## Help

- close: Hide the help menu.
- escape: Clear the filter, or hide the help menu.
- arrow
  - `n`: Move the cursor up or down n lines. Negative value for up, positive value for down.
- filter: Apply a filter for the help items.
