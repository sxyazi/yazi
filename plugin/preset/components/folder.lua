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

function Folder:window(kind) return (self:by_kind(kind) or {}).window end

function Folder:hovered(kind) return (self:by_kind(kind) or {}).hovered end

function Folder:markers(area, markers)
	if #markers == 0 then
		return {}
	end

	local elements = {}
	local append = function(last)
		local rect = ui.Rect {
			x = area.x - 1,
			y = area.y + last[1] - 1,
			w = 1,
			h = 1 + last[2] - last[1],
		}
		elements[#elements + 1] = ui.Paragraph(rect, {}):style(THEME.marker.selected)
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
	local window = self:window(self.Kind.Parent)
	if window == nil then
		return {}
	end

	local hovered = (self:hovered(self.Kind.Parent) or {}).url
	local lines = {}
	for _, f in ipairs(window) do
		local line = ui.Line { ui.Span(" " .. f:icon() .. " " .. f.name .. " ") }
		if f.url == hovered then
			line = line:style(THEME.files.hovered)
		else
			line = line:style(f:style())
		end

		lines[#lines + 1] = line
	end

	return { ui.Paragraph(area, lines) }
end

function Folder:current(area)
	local hovered = (self:hovered(self.Kind.Current) or {}).url
	local markers = {}
	local lines = {}
	for i, f in ipairs(self:window(self.Kind.Current)) do
		local name = f.name

		-- Show symlink target
		if MANAGER.show_symlink then
			local link_to = f.link_to
			if link_to ~= nil then
				name = name .. " -> " .. tostring(link_to)
			end
		end

		-- Highlight hovered file
		local line = ui.Line { ui.Span(" " .. f:icon() .. " " .. name .. " ") }
		if f.url == hovered then
			line = line:style(THEME.files.hovered)
		else
			line = line:style(f:style())
		end
		lines[#lines + 1] = line

		-- Mark selected/yanked files
		if f:selected() then
			markers[#markers + 1] = { i, 1 }
		end
	end

	return { ui.Paragraph(area, lines), table.unpack(self:markers(area, markers)) }
end

function Folder:preview(area)
	local window = self:window(self.Kind.Preview)
	if window == nil then
		return {}
	end

	local hovered = (self:hovered(self.Kind.Preview) or {}).url
	local lines = {}
	for _, f in ipairs(window) do
		local line = ui.Line { ui.Span(" " .. f:icon() .. " " .. f.name .. " ") }
		if f.url == hovered then
			line = line:style(THEME.preview.hovered)
		else
			line = line:style(f:style())
		end
		lines[#lines + 1] = line
	end

	return { ui.Paragraph(area, lines) }
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
