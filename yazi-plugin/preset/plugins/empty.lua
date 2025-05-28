local M = {}

function M.msg(job, s) ya.preview_widget(job, ui.Text(ui.Line(s):reverse()):area(job.area):wrap(ui.Wrap.YES)) end

function M:peek(job)
	local path = tostring(job.file.url)
	if path:sub(1, 6) ~= "/proc/" then
		return self.msg(job, "Empty file")
	end

	local i, j, lines = 0, 0, {}
	local file = io.open(path, "r")
	if not file then
		return self.msg(job, "Failed to open file")
	end

	local limit = job.area.h
	while true do
		local chunk = file:read(4096)
		if not chunk then
			break
		end

		j = j + #chunk
		if j > 5242880 then
			return self.msg(job, "File too large")
		end

		for line in chunk:gmatch("[^\n]*\n?") do
			i = i + 1
			if i > job.skip + limit then
				break
			elseif i > job.skip then
				lines[#lines + 1] = line
			end
		end
	end

	if job.skip > 0 and i < job.skip + limit then
		ya.emit("peek", { math.max(0, i - limit), only_if = job.file.url, upper_bound = true })
	else
		ya.preview_widget(job, ui.Text(lines):area(job.area))
	end
end

function M:seek(job) require("code"):seek(job) end

return M
