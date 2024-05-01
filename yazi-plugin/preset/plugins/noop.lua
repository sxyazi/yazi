local M = {}

function M:peek() end

function M:seek() end

---@return yazi.PreloaderReturnValue
function M:preload() return 1 end

return M
