Entity = {
	_inc = 1000,
}

function Entity:style(file)
	local style = file:style()
	if not file:is_hovered() then
		return style
	elseif file:in_preview() then
		return style and style:patch(THEME.manager.preview_hovered) or THEME.manager.preview_hovered
	else
		return style and style:patch(THEME.manager.hovered) or THEME.manager.hovered
	end
end

function Entity:icon(file)
	local icon = file:icon()
	if not icon then
		return ui.Line("")
	elseif file:is_hovered() then
		return ui.Line(" " .. icon.text .. " ")
	else
		return ui.Line(" " .. icon.text .. " "):style(icon.style)
	end
end

function Entity:prefix(file)
	local prefix = file:prefix() or ""
	return ui.Line(prefix ~= "" and prefix .. "/" or "")
end

function Entity:highlights(file)
	local name = file.name:gsub("\r", "?", 1)
	local highlights = file:highlights()
	if not highlights or #highlights == 0 then
		return ui.Line(name)
	end

	local spans, last = {}, 0
	for _, h in ipairs(highlights) do
		if h[1] > last then
			spans[#spans + 1] = ui.Span(name:sub(last + 1, h[1]))
		end
		spans[#spans + 1] = ui.Span(name:sub(h[1] + 1, h[2])):style(THEME.manager.find_keyword)
		last = h[2]
	end
	if last < #name then
		spans[#spans + 1] = ui.Span(name:sub(last + 1))
	end
	return ui.Line(spans)
end

function Entity:found(file)
	if not file:is_hovered() then
		return ui.Line {}
	end

	local found = file:found()
	if not found then
		return ui.Line {}
	end

	return ui.Line {
		ui.Span("  "),
		ui.Span(string.format("[%d/%d]", found[1] + 1, found[2])):style(THEME.manager.find_position),
	}
end

function Entity:symlink(file)
	if not MANAGER.show_symlink then
		return ui.Line {}
	end

	local to = file.link_to
	return ui.Line(to and { ui.Span(" -> " .. tostring(to)):italic() } or {})
end

function Entity:render(file)
	local lines = {}
	for _, child in ipairs(self._children) do
		lines[#lines + 1] = child[1](self, file)
	end
	return ui.Line(lines)
end

-- Initialize children
Entity._children = {
	{ Entity.icon, id = 1, order = 1000 },
	{ Entity.prefix, id = 2, order = 2000 },
	{ Entity.highlights, id = 3, order = 3000 },
	{ Entity.found, id = 4, order = 4000 },
	{ Entity.symlink, id = 5, order = 5000 },
}

function Entity:children_add(fn, order)
	self._inc = self._inc + 1
	self._children[#self._children + 1] = { fn, id = self._inc, order = order }
	table.sort(self._children, function(a, b) return a.order < b.order end)
	return self._inc
end

function Entity:children_remove(id)
	for i, child in ipairs(self._children) do
		if child.id == id then
			table.remove(self._children, i)
			break
		end
	end
end
