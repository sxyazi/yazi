local M = {}

function M:peek(job) ya.preview_widget(job, {}) end

function M:seek() end

function M:fetch() return true end

function M:preload() return true end

function M:spot() end

return M
