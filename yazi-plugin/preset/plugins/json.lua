local M = {}

function M:peek(job)
	local child = Command("jq")
		:args({ "-b", "-C", "--tab", ".", tostring(job.file.url) })
		:stdout(Command.PIPED)
		:stderr(Command.PIPED)
		:spawn()

	if not child then
		return require("code"):peek(job)
	end

	local limit = job.area.h
	local i, lines = 0, ""
	repeat
		local next, event = child:read_line()
		if event == 1 then
			return require("code"):peek(job)
		elseif event ~= 0 then
			break
		end

		i = i + 1
		if i > job.skip then
			lines = lines .. next
		end
	until i >= job.skip + limit

	child:start_kill()
	if job.skip > 0 and i < job.skip + limit then
		ya.manager_emit("peek", { math.max(0, i - limit), only_if = job.file.url, upper_bound = true })
	else
		lines = lines:gsub("\t", string.rep(" ", rt.preview.tab_size))
		ya.preview_widgets(job, {
			ui.Text.parse(lines):area(job.area):wrap(rt.preview.wrap == "yes" and ui.Text.WRAP or ui.Text.WRAP_NO),
		})
	end
end

function M:seek(job) require("code"):seek(job) end

return M
