Header = {
	LEFT = 0,
	RIGHT = 1,

	_id = "header",
	_inc = 1000,
}

function Header:new(area, tab)
	return setmetatable({
		_area = area,
		_tab = tab,
	}, { __index = self })
end

function Header:cwd()
	local max = self._area.w - self._right_width
	if max <= 0 then
		return ui.Span("")
	end

	local s = ya.readable_path(tostring(self._tab.current.cwd)) .. self:flags()
	return ui.Span(ya.truncate(s, { max = max, rtl = true })):style(THEME.manager.cwd)
end

function Header:flags()
	local cwd = self._tab.current.cwd
	local filter = self._tab.current.files.filter

	local s = cwd.is_search and string.format(" (search: %s", cwd:frag()) or ""
	if not filter then
		return s == "" and s or s .. ")"
	elseif s == "" then
		return string.format(" (filter: %s)", tostring(filter))
	else
		return string.format("%s, filter: %s)", s, tostring(filter))
	end
end

function Header:count()
	local yanked = #cx.yanked

	local count, style
	if yanked == 0 then
		count = #self._tab.selected
		style = THEME.manager.count_selected
	elseif cx.yanked.is_cut then
		count = yanked
		style = THEME.manager.count_cut
	else
		count = yanked
		style = THEME.manager.count_copied
	end

	if count == 0 then
		return ui.Line {}
	end

	return ui.Line {
		ui.Span(string.format(" %d ", count)):style(style),
		ui.Span(" "),
	}
end

function Header:tabs()
	local tabs = #cx.tabs
	if tabs == 1 then
		return ui.Line {}
	end

	local spans = {}
	for i = 1, tabs do
		local text = i
		if THEME.manager.tab_width > 2 then
			text = ya.truncate(text .. " " .. cx.tabs[i]:name(), { max = THEME.manager.tab_width })
		end
		if i == cx.tabs.idx then
			spans[#spans + 1] = ui.Span(" " .. text .. " "):style(THEME.manager.tab_active)
		else
			spans[#spans + 1] = ui.Span(" " .. text .. " "):style(THEME.manager.tab_inactive)
		end
	end
	return ui.Line(spans)
end

function Header:render()
	local right = self:children_render(self.RIGHT)
	self._right_width = right:width()

	local left = self:children_render(self.LEFT)
	return {
		ui.Paragraph(self._area, { left }),
		ui.Paragraph(self._area, { right }):align(ui.Paragraph.RIGHT),
	}
end

-- Mouse events
function Header:click(event, up) end

function Header:scroll(event, step) end

function Header:touch(event, step) end

-- Initialize children
Header._left = {
	{ Header.cwd, id = 1, order = 1000 },
}
Header._right = {
	{ Header.count, id = 1, order = 1000 },
	{ Header.tabs, id = 2, order = 2000 },
}

function Header:children_add(fn, order, side)
	self._inc = self._inc + 1
	local children = side == self.RIGHT and self._right or self._left

	children[#children + 1] = { fn, id = self._inc, order = order }
	table.sort(children, function(a, b) return a.order < b.order end)

	return self._inc
end

function Header:children_remove(id, side)
	local children = side == self.RIGHT and self._right or self._left
	for i, child in ipairs(children) do
		if child.id == id then
			table.remove(children, i)
			break
		end
	end
end

function Header:children_render(side)
	local lines = {}
	for _, child in ipairs(side == self.RIGHT and self._right or self._left) do
		lines[#lines + 1] = child[1](self)
	end
	return ui.Line(lines)
end
