---@class yazi.Current
---@field area? unknown
Current = {
	area = ui.Rect.default,
}

---@param area unknown
---@return table
function Current:empty(area)
	local folder = Folder:by_kind(Folder.CURRENT)

	local line
	if folder.files.filter then
		line = ui.Line("No filter results")
	else
		line = ui.Line(folder.stage == "loading" and "Loading..." or "No items")
	end

	return {
		ui.Paragraph(area, { line }):align(ui.Paragraph.CENTER),
	}
end

---@param area unknown
---@return table
function Current:render(area)
	self.area = area

	local files = Folder:by_kind(Folder.CURRENT).window
	if #files == 0 then
		return self:empty(area)
	end

	local items, markers = {}, {}
	for i, f in ipairs(files) do
		items[#items + 1] = ui.ListItem(ui.Line(File:full(f))):style(File:style(f))

		-- Yanked/marked/selected files
		local marker = File:marker(f)
		if marker ~= 0 then
			markers[#markers + 1] = { i, marker }
		end
	end

	return ya.flat {
		ui.List(area, items),
		Folder:linemode(area, files),
		Folder:markers(area, markers),
	}
end
