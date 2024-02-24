Parent = {
	area = ui.Rect.default,
}

function Parent:render(area)
	self.area = area

	local folder = Folder:by_kind(Folder.PARENT)
	if not folder then
		return {}
	end

	local items, markers = {}, {}
	for i, f in ipairs(folder.window) do
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
		Folder:markers(area, markers),
	}
end
