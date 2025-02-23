Header = {
	LEFT = 0,
	RIGHT = 1,

	_id = "header",
	_inc = 1000,
	_left = {
		{ "cwd", id = 1, order = 1000 },
	},
	_right = {
		{ "count", id = 1, order = 1000 },
		{ "tabs", id = 2, order = 2000 },
	},
}

function Header:new(area, tab)
	return setmetatable({
		_area = area,
		_tab = tab,
		_current = tab.current,
	}, { __index = self })
end

function Header:cwd()
	local max = self._area.w - self._right_width
	if max <= 0 then
		return ""
	end

	local s = ya.readable_path(tostring(self._current.cwd)) .. self:flags()
	return ui.Span(ya.truncate(s, { max = max, rtl = true })):style(th.manager.cwd)
end

function Header:flags()
	local cwd = self._current.cwd
	local filter = self._current.files.filter
	local finder = self._tab.finder

	local t = {}
	if cwd.is_search then
		t[#t + 1] = string.format("search: %s", cwd:frag())
	end
	if filter then
		t[#t + 1] = string.format("filter: %s", filter)
	end
	if finder then
		t[#t + 1] = string.format("find: %s", finder)
	end
	return #t == 0 and "" or " (" .. table.concat(t, ", ") .. ")"
end

function Header:count()
	local yanked = #cx.yanked

	local count, style
	if yanked == 0 then
		count = #self._tab.selected
		style = th.manager.count_selected
	elseif cx.yanked.is_cut then
		count = yanked
		style = th.manager.count_cut
	else
		count = yanked
		style = th.manager.count_copied
	end

	if count == 0 then
		return ""
	end

	return ui.Line {
		ui.Span(string.format(" %d ", count)):style(style),
		" ",
	}
end

function Header:tabs()
	local tabs = #cx.tabs
	if tabs == 1 then
		return ""
	end

	local spans = {}
	for i = 1, tabs do
		local text = i
		if th.manager.tab_width > 2 then
			text = ya.truncate(text .. " " .. cx.tabs[i]:name(), { max = th.manager.tab_width })
		end
		if i == cx.tabs.idx then
			spans[#spans + 1] = ui.Span(" " .. text .. " "):style(th.manager.tab_active)
		else
			spans[#spans + 1] = ui.Span(" " .. text .. " "):style(th.manager.tab_inactive)
		end
	end
	return ui.Line(spans)
end

function Header:reflow() return { self } end

function Header:redraw()
	local right = self:children_redraw(self.RIGHT)
	self._right_width = right:width()

	local left = self:children_redraw(self.LEFT)

	return {
		ui.Text(left):area(self._area),
		ui.Text(right):area(self._area):align(ui.Text.RIGHT),
	}
end

-- Mouse events
function Header:click(event, up) end

function Header:scroll(event, step) end

function Header:touch(event, step) end

-- Children
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

function Header:children_redraw(side)
	local lines = {}
	for _, c in ipairs(side == self.RIGHT and self._right or self._left) do
		lines[#lines + 1] = (type(c[1]) == "string" and self[c[1]] or c[1])(self)
	end
	return ui.Line(lines)
end
