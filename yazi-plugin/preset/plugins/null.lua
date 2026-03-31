local M = {}

function M:peek(job)
	local err
	if job.mime == "null/file1-not-found" then
		local s = ya.target_family() == "windows" and "Set it up correctly as per the Windows Installation Guide"
			or "Make sure it's installed and restart Yazi"
		err = string.format("Cannot find `%s` to detect the file's MIME type. %s.", require("mime.local").file1_bin(), s)
	else
		err = "Unknown error occurred while detecting MIME type"
	end

	local line = ui.Line(err):reverse()
	ya.preview_widget(job, ui.Text(line):area(job.area):wrap(ui.Wrap.YES))
end

function M:seek() end

function M:spot(job) require("file"):spot(job) end

return M
