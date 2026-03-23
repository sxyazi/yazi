local M = {}

function M:peek(job)
	local child = Command("jq")
		:arg({ "-b", "-C", "--tab", ".", tostring(job.file.path) })
		:stdout(Command.PIPED)
		:stderr(Command.PIPED)
		:spawn()

	if not child then
		return require("code"):peek(job)
	end

	local opt = { ansi = true, tab_size = rt.preview.tab_size, wrap = rt.preview.wrap, width = job.area.w }
	local limit = job.area.h
	local i, lines = 0, {}
	repeat
		local next, event = child:read_line()
		if event == 1 then
			return require("code"):peek(job)
		elseif event ~= 0 then
			break
		end

		local wrapped = ui.lines(next, opt)
		local from = math.max(1, job.skip - i + 1)
		local to = math.min(#wrapped, job.skip + limit - i)

		i = i + #wrapped
		for j = from, to do
			lines[#lines + 1] = wrapped[j]
		end
	until i >= job.skip + limit

	child:start_kill()
	if job.skip > 0 and i < job.skip + limit then
		ya.emit("peek", { math.max(0, i - limit), only_if = job.file.url, upper_bound = true })
	else
		ya.preview_widget(job, ui.Text(lines):area(job.area))
	end
end

function M:seek(job) require("code"):seek(job) end

return M
