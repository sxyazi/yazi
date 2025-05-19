Entity = {
	_inc = 1000,
	_children = {
		{ "spacer", id = 1, order = 1000 },
		{ "icon", id = 2, order = 2000 },
		{ "prefix", id = 3, order = 3000 },
		{ "highlights", id = 4, order = 4000 },
		{ "found", id = 5, order = 5000 },
		{ "symlink", id = 6, order = 6000 },
	},
}

function Entity:new(file) return setmetatable({ _file = file }, { __index = self }) end

function Entity:spacer() return " " end

function Entity:icon()
	local icon = self._file:icon()
	if not icon then
		return ""
	elseif self._file.is_hovered then
		return icon.text .. " "
	else
		return ui.Line(icon.text .. " "):style(icon.style)
	end
end

function Entity:prefix()
	local prefix = self._file:prefix() or ""
	return prefix ~= "" and prefix .. "/" or ""
end

function Entity:highlights()
	local name = self._file.name:gsub("\r", "?", 1)
	local highlights = self._file:highlights()
	if not highlights or #highlights == 0 then
		return name
	end

	local spans, last = {}, 0
	for _, h in ipairs(highlights) do
		if h[1] > last then
			spans[#spans + 1] = name:sub(last + 1, h[1])
		end
		spans[#spans + 1] = ui.Span(name:sub(h[1] + 1, h[2])):style(th.mgr.find_keyword)
		last = h[2]
	end
	if last < #name then
		spans[#spans + 1] = name:sub(last + 1)
	end
	return ui.Line(spans)
end

function Entity:found()
	if not self._file.is_hovered then
		return ""
	end

	local found = self._file:found()
	if not found then
		return ""
	elseif found[1] >= 99 then
		return ""
	end

	local s = string.format("[%d/%s]", found[1] + 1, found[2] >= 100 and "99+" or found[2])
	return ui.Line { "  ", ui.Span(s):style(th.mgr.find_position) }
end

function Entity:symlink()
	if not rt.mgr.show_symlink then
		return ""
	end

	local to = self._file.link_to
	return to and ui.Span(string.format(" -> %s", to)):style(th.mgr.symlink_target) or ""
end

function Entity:redraw()
	local lines = {}
	for _, c in ipairs(self._children) do
		local line = (type(c[1]) == "string" and self[c[1]] or c[1])(self)
		c.width, lines[#lines + 1] = ui.width(line), line
	end
	return ui.Line(lines):style(self:style())
end

function Entity:style()
	local s = self._file:style()
	if not self._file.is_hovered then
		return s
	elseif self._file.in_preview then
		return s and s:patch(th.mgr.preview_hovered) or th.mgr.preview_hovered
	else
		return s and s:patch(th.mgr.hovered) or th.mgr.hovered
	end
end

function Entity:ellipsis(max)
	local adv, f = 0, self._file
	for _, child in ipairs(self._children) do
		adv = adv + child.width
		if adv >= max then
			return not f.cha.is_dir and f.url.ext and "…." .. f.url.ext or nil
		elseif child.id == 4 then
			break
		end
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
