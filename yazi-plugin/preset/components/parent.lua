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
		-- Highlight hovered file
		local item = ui.ListItem(ui.Line { Folder:icon(f), ui.Span(f.name) })
		if f:is_hovered() then
			item = item:style(THEME.manager.hovered)
		else
			item = item:style(f:style())
		end
		items[#items + 1] = item

		-- Yanked/marked/selected files
		local marker = Folder:marker(f)
		if marker ~= 0 then
			markers[#markers + 1] = { i, marker }
		end
	end

	return ya.flat {
		ui.List(area, items),
		Folder:markers(area, markers),
	}
end
