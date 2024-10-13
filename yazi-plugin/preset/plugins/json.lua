local M = {}

function M:peek()
	local child = Command("jq")
		:args({
			"-C",
			"--tab",
			".",
			tostring(self.file.url),
		})
		:stdout(Command.PIPED)
		:stderr(Command.PIPED)
		:spawn()

	if not child then
		return require("code").peek(self)
	end

	local limit = self.area.h
	local i, lines = 0, ""
	repeat
		local next, event = child:read_line()
		if event == 1 then
			return require("code").peek(self)
		elseif event ~= 0 then
			break
		end

		i = i + 1
		if i > self.skip then
			lines = lines .. next
		end
	until i >= self.skip + limit

	child:start_kill()
	if self.skip > 0 and i < self.skip + limit then
		ya.manager_emit("peek", { math.max(0, i - limit), only_if = self.file.url, upper_bound = true })
	else
		lines = lines:gsub("\t", string.rep(" ", PREVIEW.tab_size))
		ya.preview_widgets(self, { ui.Text.parse(lines):area(self.area) })
	end
end

function M:seek(units) require("code").seek(self, units) end

return M
