Current = {
	area = ui.Rect.default,
}

function Current:render(area)
	self.area = area

	local markers = {}
	local items = {}
	for i, f in ipairs(Folder:by_kind(Folder.CURRENT).window) do
		local name = Folder:highlighted_name(f)

		-- Highlight hovered file
		local item = ui.ListItem(ui.Line { Folder:icon(f), table.unpack(name) })
		if f:is_hovered() then
			item = item:style(THEME.manager.hovered)
		else
			item = item:style(f:style())
		end
		items[#items + 1] = item

		-- Mark yanked/selected files
		local yanked = f:is_yanked()
		if yanked ~= 0 then
			markers[#markers + 1] = { i, yanked }
		elseif f:is_selected() then
			markers[#markers + 1] = { i, 3 }
		end
	end
	return ya.flat { ui.List(area, items), Folder:linemode(area), Folder:markers(area, markers) }
end
