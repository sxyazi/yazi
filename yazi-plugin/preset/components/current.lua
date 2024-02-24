Current = {
	area = ui.Rect.default,
}

function Current:render(area)
	self.area = area

	local files = Folder:by_kind(Folder.CURRENT).window
	if #files == 0 then
		return {}
	end

	local items, markers = {}, {}
	for i, f in ipairs(files) do
		local style = f:style()
		items[#items + 1] = ui.ListItem(ui.Line(File:full(f)))
			:style(f:is_hovered() and style:patch(THEME.manager.hovered) or style)

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
