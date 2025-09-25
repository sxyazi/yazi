local M = {}

function M:peek(job)
	local line = ui.Line("Remote file, download to preview"):reverse()
	ya.preview_widget(job, ui.Text(line):area(job.area):wrap(ui.Wrap.YES))
end

function M:seek() end

function M:spot(job) require("file"):spot(job) end

return M
