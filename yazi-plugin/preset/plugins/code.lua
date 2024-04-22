local M = {}

function M:peek()
	local _, bound = ya.preview_code(self)
	if bound then
		ya.manager_emit("peek", { bound, only_if = self.file.url, upper_bound = true })
	end
end

function M:seek(units)
	local h = cx.active.current.hovered
	if h and h.url == self.file.url then
		local step = math.floor(units * self.area.h / 10)
		ya.manager_emit("peek", {
			math.max(0, cx.active.preview.skip + step),
			only_if = self.file.url,
		})
	end
end

return M
