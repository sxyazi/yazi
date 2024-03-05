local M = {}

function M:peek()
	local folder = Folder:by_kind(Folder.PREVIEW)
	if not folder or folder.cwd ~= self.file.url then
		return {}
	end

	local bound = math.max(0, #folder.files - self.area.h)
	if self.skip > bound then
		ya.manager_emit("peek", { bound, only_if = tostring(self.file.url), upper_bound = true })
	end

	local items, markers = {}, {}
	for i, f in ipairs(folder.window) do
		items[#items + 1] = ui.ListItem(ui.Line(File:full(f))):style(File:style(f))

		-- Yanked/marked/selected files
		local marker = File:marker(f)
		if marker ~= 0 then
			markers[#markers + 1] = { i, marker }
		end
	end

	ya.preview_widgets(
		self,
		ya.flat {
			ui.List(self.area, items),
			Folder:markers(self.area, markers),
		}
	)
end

function M:seek(units)
	local folder = Folder:by_kind(Folder.PREVIEW)
	if folder and folder.cwd == self.file.url then
		local step = math.floor(units * self.area.h / 10)
		local bound = math.max(0, #folder.files - self.area.h)
		ya.manager_emit("peek", {
			ya.clamp(0, cx.active.preview.skip + step, bound),
			only_if = tostring(self.file.url),
		})
	end
end

return M
