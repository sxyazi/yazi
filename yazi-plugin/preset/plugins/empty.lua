local M = {}

function M:msg(s) ya.preview_widgets(self, { ui.Text(ui.Line(s):reverse()):area(self.area):wrap(ui.Text.WRAP) }) end

function M:peek()
	local path = tostring(self.file.url)
	if path:sub(1, 6) ~= "/proc/" then
		return self:msg("Empty file")
	end

	local limit = self.area.h
	local i, lines = 0, {}
	local ok, err = pcall(function()
		for line in io.lines(path) do
			i = i + 1
			if i > self.skip + limit then
				break
			elseif i > self.skip then
				lines[#lines + 1] = ui.Line(line)
			end
		end
	end)

	if not ok then
		self:msg(err)
	elseif self.skip > 0 and i < self.skip + limit then
		ya.manager_emit("peek", { math.max(0, i - limit), only_if = self.file.url, upper_bound = true })
	else
		ya.preview_widgets(self, { ui.Text(lines):area(self.area) })
	end
end

function M:seek(units) require("code").seek(self, units) end

return M
