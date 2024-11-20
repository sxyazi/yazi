local M = {}

function M:peek()
	local folder = cx.active.preview.folder
	if not folder or folder.cwd ~= self.file.url then
		return
	end

	local bound = math.max(0, #folder.files - self.area.h)
	if self.skip > bound then
		return ya.manager_emit("peek", { bound, only_if = self.file.url, upper_bound = true })
	end

	if #folder.files == 0 then
		return ya.preview_widgets(self, {
			ui.Text(folder.stage.is_loading and "Loading..." or "No items"):area(self.area):align(ui.Text.CENTER),
		})
	end

	local entities = {}
	for _, f in ipairs(folder.window) do
		entities[#entities + 1] = Entity:new(f):redraw()
	end

	ya.preview_widgets(self, {
		ui.List(entities):area(self.area),
		table.unpack(Marker:new(self.area, folder):redraw()),
	})
end

function M:seek(units)
	local folder = cx.active.preview.folder
	if folder and folder.cwd == self.file.url then
		local step = math.floor(units * self.area.h / 10)
		local bound = math.max(0, #folder.files - self.area.h)
		ya.manager_emit("peek", {
			ya.clamp(0, cx.active.preview.skip + step, bound),
			only_if = self.file.url,
		})
	end
end

function M:spot(args) require("file"):spot(args) end

return M
