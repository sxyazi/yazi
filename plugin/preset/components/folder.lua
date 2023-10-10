Folder = {
	Kind = {
		Parent = 0,
		Current = 1,
		Preview = 2,
	},
}

function Folder:by_kind(kind)
	if kind == self.Kind.Parent then
		return cx.active.parent
	elseif kind == self.Kind.Current then
		return cx.active.current
	elseif kind == self.Kind.Preview then
		return cx.active.preview.folder
	end
end

function Folder:markers(area, markers)
	if #markers == 0 then
		return {}
	end

	local elements = {}
	local append = function(last)
		local p = ui.Paragraph(
			ui.Rect {
				x = area.x - 1,
				y = area.y + last[1] - 1,
				w = 1,
				h = 1 + last[2] - last[1],
			},
			{}
		)

		if last[3] == 1 then
			p = p:style(THEME.marker.copied)
		elseif last[3] == 2 then
			p = p:style(THEME.marker.cut)
		elseif last[3] == 3 then
			p = p:style(THEME.marker.selected)
		end
		elements[#elements + 1] = p
	end

	local last = { markers[1][1], markers[1][1], markers[1][2] } -- start, end, type
	for _, m in ipairs(markers) do
		if m[1] - last[2] > 1 or last[3] ~= m[2] then
			append(last)
			last = { m[1], m[1], m[2] }
		else
			last[2] = m[1]
		end
	end

	append(last)
	return elements
end

function Folder:parent(area)
	local folder = self:by_kind(self.Kind.Parent)
	if folder == nil then
		return {}
	end

	local items = {}
	for _, f in ipairs(folder.window) do
		local item = ui.ListItem(" " .. f:icon() .. " " .. f.name .. " ")
		if f.hovered then
			item = item:style(THEME.files.hovered)
		else
			item = item:style(f:style())
		end

		items[#items + 1] = item
	end

	return { ui.List(area, items) }
end

function Folder:current(area)
	local markers = {}
	local items = {}
	for i, f in ipairs(self:by_kind(self.Kind.Current).window) do
		local name = ui.highlight_ranges(f.name, f:highlights())

		-- Show symlink target
		if MANAGER.show_symlink then
			if f.link_to ~= nil then
				name[#name + 1] = ui.Span(" -> " .. tostring(f.link_to)):italic()
			end
		end

		-- Highlight hovered file
		local item = ui.ListItem(ui.Line { ui.Span(" " .. f:icon() .. " "), table.unpack(name) })
		if f.hovered then
			item = item:style(THEME.files.hovered)
		else
			item = item:style(f:style())
		end
		items[#items + 1] = item

		-- Mark yanked/selected files
		local yanked = f:yanked()
		if yanked ~= 0 then
			markers[#markers + 1] = { i, yanked }
		elseif f:selected() then
			markers[#markers + 1] = { i, 3 }
		end
	end

	return { ui.List(area, items), table.unpack(self:markers(area, markers)) }
end

function Folder:preview(area)
	local folder = self:by_kind(self.Kind.Preview)
	if folder == nil then
		return {}
	end

	local items = {}
	for _, f in ipairs(folder.window) do
		local item = ui.ListItem(" " .. f:icon() .. " " .. f.name .. " ")
		if f.hovered then
			item = item:style(THEME.preview.hovered)
		else
			item = item:style(f:style())
		end
		items[#items + 1] = item
	end

	return { ui.List(area, items) }
end

function Folder:render(area, args)
	if args.kind == self.Kind.Parent then
		return self:parent(area)
	elseif args.kind == self.Kind.Current then
		return self:current(area)
	elseif args.kind == self.Kind.Preview then
		return self:preview(area)
	end
end
