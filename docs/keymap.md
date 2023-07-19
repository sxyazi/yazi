# Keymap

## manager

- escape: Exit visual mode, clear selected, or cancel search.
- quit: Exit the process.
- close: Close the current tab; if it is the last tab, then exit the process.

Navigation

- arrow

  - `n`: Move the cursor up or down by n lines. Use negative values to move up and positive values to move down.

- leave: Go back to the parent directory.
- enter: Enter the child directory.
- back: Go back to the previous directory.
- forward: Go forward to the next directory.

Selection

- select

  - `--state=true`: Select the current file.
  - `--state=false`: Deselect the current file.
  - `--state=none`: Default, toggle the selection state of the current file.

- visual_mode: Enter visual mode (selection mode).

  - `--unselect`: Enter visual mode (deselect mode).

- select_all

  - `--state=true`: Select all files.
  - `--state=false`: Deselect all files.
  - `--state=none`: Default, toggle the selection state of all files.

Operation

- open: Open the selected files.

  - `--select`: Open the selected files with an interactive ui to choose the opening method.

- yank: Copy the selected files.

  - `--cut`: Cut the selected files.

- paste: Paste the files.

  - `--force`: Overwrite the destination file if it exists.
  - `--follow`: Copy the file pointed to by a symbolic link, rather than the link itself. Only valid during copying.

- remove: Move the file to the trash/recycle bin.

  - `--permanently`: Permanently delete the file.

- create: Create a file or directory (append `/` at the end of the filename for directory).

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

Tabs

- tab_create

  - `path`: Create a new tab using the specified path.
  - `--current`: Create a new tab based on the current directory.

- tab_close

  - `n`: Close the tab at position n, starting from 0.

- tab_switch

  - `n`: Switch to the tab at position n, starting from 0.
  - `relative`: Switch to the tab at a position relative to the current tab. The value of n can be negative when using this parameter.

- tab_swap

  - `n`: Swap the current tab with the tab at position n, where negative values move the tab forward, and positive values move it backward.

Tasks

- tasks_show: Display the task manager.

## tasks

TODO

## select

TODO

## input

TODO
