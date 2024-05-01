---@meta

-- This file contains some of the types used in yazi. It only exists to
-- provide these types and should not be loaded at runtime.
return

---@alias yazi.FolderType "PARENT" | "CURRENT" | "PREVIEW"

---@alias yazi.PreloaderReturnValue
---| 0 # Failure, don't continue
---| 1 # Success, don't continue
---| 2 # Failure, continue
---| 3 # Success, continue
