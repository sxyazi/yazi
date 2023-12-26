Parent = {
	area = ui.Rect.default,
}

function Parent:render(area)
	self.area = area

	local folder = Folder:by_kind(Folder.PARENT)
	if folder == nil then
		return {}
	end

	local items = {}
	for _, f in ipairs(folder.window) do
		local item = ui.ListItem(ui.Line { Folder:icon(f), ui.Span(f.name) })
		if f:is_hovered() then
			item = item:style(THEME.manager.hovered)
		else
			item = item:style(f:style())
		end

		items[#items + 1] = item
	end

	return { ui.List(area, items) }
end
