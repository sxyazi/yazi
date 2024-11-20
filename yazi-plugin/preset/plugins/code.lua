local M = {}

function M:peek()
	local err, bound = ya.preview_code(self)
	if bound then
		ya.manager_emit("peek", { bound, only_if = self.file.url, upper_bound = true })
	elseif err and not err:find("cancelled", 1, true) then
		ya.preview_widgets(self, {
			ui.Text(err):area(self.area):reverse(),
		})
	end
end

function M:seek(units)
	local h = cx.active.current.hovered
	if not h or h.url ~= self.file.url then
		return
	end

	local step = math.floor(units * self.area.h / 10)
	step = step == 0 and ya.clamp(-1, units, 1) or step

	ya.manager_emit("peek", {
		math.max(0, cx.active.preview.skip + step),
		only_if = self.file.url,
	})
end

function M:spot(args) require("file"):spot(args) end

return M
