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
		local name = Folder:highlighted_name(f)

		-- Highlight hovered file
		local item = ui.ListItem(ui.Line { Folder:icon(f), table.unpack(name) })
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
		Folder:linemode(area, files),
		Folder:markers(area, markers),
	}
end
