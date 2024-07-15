Entity = {
	_inc = 1000,
	_children = {
		{ "icon", id = 1, order = 1000 },
		{ "prefix", id = 2, order = 2000 },
		{ "highlights", id = 3, order = 3000 },
		{ "found", id = 4, order = 4000 },
		{ "symlink", id = 5, order = 5000 },
	},
}

function Entity:new(file) return setmetatable({ _file = file }, { __index = self }) end

function Entity:icon()
	local icon = self._file:icon()
	if not icon then
		return ui.Line("")
	elseif self._file:is_hovered() then
		return ui.Line(" " .. icon.text .. " ")
	else
		return ui.Line(" " .. icon.text .. " "):style(icon.style)
	end
end

function Entity:prefix()
	local prefix = self._file:prefix() or ""
	return ui.Line(prefix ~= "" and prefix .. "/" or "")
end

function Entity:highlights()
	local name = self._file.name:gsub("\r", "?", 1)
	local highlights = self._file:highlights()
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

function Entity:found()
	if not self._file:is_hovered() then
		return ui.Line {}
	end

	local found = self._file:found()
	if not found then
		return ui.Line {}
	end

	return ui.Line {
		ui.Span("  "),
		ui.Span(string.format("[%d/%d]", found[1] + 1, found[2])):style(THEME.manager.find_position),
	}
end

function Entity:symlink()
	if not MANAGER.show_symlink then
		return ui.Line {}
	end

	local to = self._file.link_to
	return ui.Line(to and { ui.Span(" -> " .. tostring(to)):italic() } or {})
end

function Entity:render()
	local lines = {}
	for _, c in ipairs(self._children) do
		lines[#lines + 1] = (type(c[1]) == "string" and self[c[1]] or c[1])(self)
	end
	return ui.Line(lines)
end

function Entity:style()
	local s = self._file:style()
	if not self._file:is_hovered() then
		return s
	elseif self._file:in_preview() then
		return s and s:patch(THEME.manager.preview_hovered) or THEME.manager.preview_hovered
	else
		return s and s:patch(THEME.manager.hovered) or THEME.manager.hovered
	end
end

-- Children
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
