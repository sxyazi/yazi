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
- cd: Change the current directory.

  - `path`: the path to change to.

### Selection

- select

  - `--state=true`: Select the current file.
  - `--state=false`: Deselect the current file.
  - `--state=none`: Default, toggle the selection state of the current file.

- visual_mode: Enter visual mode (selection mode).

  - `--unset`: Enter visual mode (unset mode).

- select_all

  - `--state=true`: Select all files.
  - `--state=false`: Deselect all files.
  - `--state=none`: Default, toggle the selection state of all files.

### Operation

- open: Open the selected files.

  - `--select`: Open the selected files with an interactive ui to choose the opening method.

- yank: Copy the selected files.

  - `--cut`: Cut the selected files.

- paste: Paste the files.

  - `--force`: Overwrite the destination file if it exists.
  - `--follow`: Copy the file pointed to by a symbolic link, rather than the link itself. Only valid during copying.

- remove: Move the file to the trash/recycle bin.

  - `--permanently`: Permanently delete the file.

- create: Create a file or directory (ends with `/` for directory).
- rename: Rename a file or directory.
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

### Tabs

- tab_create

  - `path`: Create a new tab using the specified path.
  - `--current`: Create a new tab based on the current directory.

- tab_close

  - `n`: Close the tab at position n, starting from 0.

- tab_switch

  - `n`: Switch to the tab at position n, starting from 0.
  - `--relative`: Switch to the tab at a position relative to the current tab. The value of n can be negative when using this parameter.

- tab_swap

  - `n`: Swap the current tab with the tab at position n, where negative values move the tab forward, and positive values move it backward.

### Tasks

- tasks_show: Display the task manager.

## tasks

- close: Hide the task manager.
- arrow:
  - `-1`: Move the cursor up 1 line.
  - `1`: Move the cursor down 1 line.
- cancel: Cancel the task.

## select

- close: Cancel selection.
  - `--submit`: Submit the selection.
- arrow
  - `n`: Move the cursor up or down n lines. Negative value for up, positive value for down.

## input

- close: Cancel input.

  - `--submit`: Submit the input.

- escape: Cancel visual mode and enter normal mode.
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

  - `--insert`: Delete and enter insert mode.

- yank: Copy the selected characters.
- paste: Paste the copied characters after the cursor.

  - `--before`: Paste the copied characters before the cursor.

- undo: Undo the last operation.
- redo: Redo the last operation.

### Insert mode

- close: Cancel input.

  - `--submit`: Submit the input.

- escape: Cancel insert mode and enter normal mode.
- backspace: Delete the character before the cursor.
