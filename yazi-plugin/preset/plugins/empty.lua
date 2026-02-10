local M = {}

function M.msg(job, s) ya.preview_widget(job, ui.Text(ui.Line(s):reverse()):area(job.area):wrap(ui.Wrap.YES)) end

function M:peek(job)
	if not job.file.url:starts_with("/proc/") then
		return self.msg(job, "Empty file")
	end

	local fd, err = fs.access():read(true):open(job.file.url)
	if not fd then
		return self.msg(job, "Failed to open file: " .. err)
	end

	local lines, err = M.read_up_to(fd, job.skip, job.area.h)
	ya.drop(fd)

	if not lines then
		self.msg(job, tostring(err))
	elseif lines.n == 0 then
		self.msg(job, "Empty file")
	elseif job.skip > 0 and lines.n < job.skip + job.area.h then
		ya.emit("peek", { math.max(0, lines.n - job.area.h), only_if = job.file.url, upper_bound = true })
	else
		ya.preview_widget(job, ui.Text(lines):area(job.area))
	end
end

function M:seek(job) require("code"):seek(job) end

--- @param fd Fd
--- @param skip integer
--- @param limit integer
--- @return { [integer]: string, n: integer }?
--- @return Error?
function M.read_up_to(fd, skip, limit)
	local seen, lines = 0, { n = 0 }
	while true do
		local chunk, err = fd:read(4096)
		if not chunk then
			return nil, Err("Failed to read file: %s", err)
		elseif chunk == "" then
			break
		end

		seen = seen + #chunk
		if seen > 5242880 then
			return nil, Err("File too large")
		end

		for line in chunk:gmatch("[^\n]*\n?") do
			lines.n = lines.n + 1
			if lines.n > skip + limit then
				break
			elseif lines.n > skip then
				lines[#lines + 1] = line
			end
		end
	end
	return lines
end

return M
