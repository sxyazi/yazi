local M = {}

function M:peek(job)
	local err
	if job.mime == "null/file1-not-found" then
		err = string.format(
			"Cannot find `%s` to detect the file's MIME type. Make sure it's installed and restart Yazi",
			require("mime.local").file1_bin()
		)
	else
		err = "Unknown error occurred while detecting MIME type"
	end

	local line = ui.Line(err):reverse()
	ya.preview_widget(job, ui.Text(line):area(job.area):wrap(ui.Wrap.YES))
end

function M:seek() end

function M:spot(job) require("file"):spot(job) end

return M
