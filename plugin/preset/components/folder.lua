Folder = {
	Kind = {
		Parent = 0,
		Current = 1,
		Preview = 2,
	},
}

function Folder:by_kind(kind)
	if kind == self.Kind.Parent then
		return cx.manager.parent
	elseif kind == self.Kind.Current then
		return cx.manager.current
	elseif kind == self.Kind.Preview then
		return cx.manager.preview.folder
	end
end

function Folder:window(kind) return (self:by_kind(kind) or {}).window end

function Folder:hovered(kind) return (self:by_kind(kind) or {}).hovered end

function Folder:parent(area)
	local window = self:window(self.Kind.Parent)
	if window == nil then
		return {}
	end

	local hovered = (self:hovered(self.Kind.Parent) or {}).url
	local lines = {}
	for _, f in pairs(window) do
		local line = ui.Line { ui.Span(" " .. f:icon() .. " " .. f.name .. " ") }
		if f.url == hovered then
			line = line:style(THEME.selection.hovered)
		else
			line = line:style(f:style())
		end

		lines[#lines + 1] = line
	end

	return { ui.Paragraph(area, lines) }
end

function Folder:current(area)
	local hovered = (self:hovered(self.Kind.Current) or {}).url
	local lines = {}
	for _, f in pairs(self:window(self.Kind.Current)) do
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
			line = line:style(THEME.selection.hovered)
		else
			line = line:style(f:style())
		end

		lines[#lines + 1] = line
	end

	return { ui.Paragraph(area, lines) }
end

function Folder:preview(area)
	local window = self:window(self.Kind.Preview)
	if window == nil then
		return {}
	end

	local hovered = (self:hovered(self.Kind.Preview) or {}).url
	local lines = {}
	for _, f in pairs(window) do
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
