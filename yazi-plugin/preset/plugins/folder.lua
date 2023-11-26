local Folder_ = {}

function Folder_:peek()
	local folder = Folder:by_kind(Folder.PREVIEW)
	if folder == nil then
		return {}
	end

	local items = {}
	for _, f in ipairs(folder.window) do
		local item = ui.ListItem(ui.Line { Folder:icon(f), ui.Span(f.name) })
		if f:is_hovered() then
			item = item:style(THEME.manager.preview_hovered)
		else
			item = item:style(f:style())
		end
		items[#items + 1] = item
	end
	ya.preview_widgets(self.file, self.skip, { ui.List(self.area, items) })
end

function Folder_:seek(units)
	-- TODO
end

return Folder_
