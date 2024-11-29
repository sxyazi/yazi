local M = {}

function M:msg(job, s) ya.preview_widgets(job, { ui.Text(s):area(job.area):reverse():wrap(ui.Text.WRAP) }) end

function M:peek(job)
	local path = tostring(job.file.url)
	if path:sub(1, 6) ~= "/proc/" then
		return self:msg(job, "Empty file")
	end

	local limit = job.area.h
	local i, lines = 0, {}
	local ok, err = pcall(function()
		for line in io.lines(path) do
			i = i + 1
			if i > job.skip + limit then
				break
			elseif i > job.skip then
				lines[#lines + 1] = line
			end
		end
	end)

	if not ok then
		self:msg(job, err)
	elseif job.skip > 0 and i < job.skip + limit then
		ya.manager_emit("peek", { math.max(0, i - limit), only_if = job.file.url, upper_bound = true })
	else
		ya.preview_widgets(job, { ui.Text(lines):area(job.area) })
	end
end

function M:seek(job) require("code"):seek(job) end

return M
