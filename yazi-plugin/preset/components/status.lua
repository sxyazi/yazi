Status = {
	-- TODO: remove these two constants
	LEFT = 0,
	RIGHT = 1,

	_id = "status",
	_inc = 1000,
	_left = {
		{ "mode", id = 1, order = 1000 },
		{ "size", id = 2, order = 2000 },
		{ "name", id = 3, order = 3000 },
	},
	_right = {
		{ "perm", id = 4, order = 1000 },
		{ "percent", id = 5, order = 2000 },
		{ "position", id = 6, order = 3000 },
	},
}

function Status:new(area, tab)
	return setmetatable({
		_area = area,
		_tab = tab,
		_current = tab.current,
	}, { __index = self })
end

function Status:style()
	local m = th.mode
	if self._tab.mode.is_select then
		return { main = m.select_main, alt = m.select_alt }
	elseif self._tab.mode.is_unset then
		return { main = m.unset_main, alt = m.unset_alt }
	else
		return { main = m.normal_main, alt = m.normal_alt }
	end
end

function Status:mode()
	local mode = tostring(self._tab.mode):sub(1, 3):upper()

	local style = self:style()
	return ui.Line {
		ui.Span(th.status.sep_left.open):fg(style.main.bg):bg("reset"),
		ui.Span(" " .. mode .. " "):style(style.main),
		ui.Span(th.status.sep_left.close):fg(style.main.bg):bg(style.alt.bg),
	}
end

function Status:size()
	local h = self._current.hovered
	local size = h and (h:size() or h.cha.len) or 0

	local style = self:style()
	return ui.Line {
		ui.Span(" " .. ya.readable_size(size) .. " "):style(style.alt),
		ui.Span(th.status.sep_left.close):fg(style.alt.bg),
	}
end

function Status:name()
	local h = self._current.hovered
	if not h then
		return ""
	end

	return " " .. h.name:gsub("\r", "?", 1)
end

function Status:perm()
	local h = self._current.hovered
	if not h then
		return ""
	end

	local perm = h.cha:perm()
	if not perm then
		return ""
	end

	local spans = {}
	for i = 1, #perm do
		local c = perm:sub(i, i)
		local style = th.status.perm_type
		if c == "-" or c == "?" then
			style = th.status.perm_sep
		elseif c == "r" then
			style = th.status.perm_read
		elseif c == "w" then
			style = th.status.perm_write
		elseif c == "x" or c == "s" or c == "S" or c == "t" or c == "T" then
			style = th.status.perm_exec
		end
		spans[i] = ui.Span(c):style(style)
	end
	return ui.Line(spans)
end

function Status:percent()
	local percent = 0
	local cursor = self._current.cursor
	local length = #self._current.files
	if cursor ~= 0 and length ~= 0 then
		percent = math.floor((cursor + 1) * 100 / length)
	end

	if percent == 0 then
		percent = " Top "
	elseif percent == 100 then
		percent = " Bot "
	else
		percent = string.format(" %2d%% ", percent)
	end

	local style = self:style()
	return ui.Line {
		ui.Span(" " .. th.status.sep_right.open):fg(style.alt.bg),
		ui.Span(percent):style(style.alt),
	}
end

function Status:position()
	local cursor = self._current.cursor
	local length = #self._current.files

	local style = self:style()
	return ui.Line {
		ui.Span(th.status.sep_right.open):fg(style.main.bg):bg(style.alt.bg),
		ui.Span(string.format(" %2d/%-2d ", math.min(cursor + 1, length), length)):style(style.main),
		ui.Span(th.status.sep_right.close):fg(style.main.bg):bg("reset"),
	}
end

function Status:reflow() return { self } end

function Status:redraw()
	local left = self:children_redraw(self.LEFT)

	local right = self:children_redraw(self.RIGHT)
	local right_width = right:width()

	return {
		ui.Text(""):area(self._area):style(th.status.overall),
		ui.Line(left):area(self._area),
		ui.Line(right):area(self._area):align(ui.Align.RIGHT),
		table.unpack(ui.redraw(Progress:new(self._area, right_width))),
	}
end

-- Mouse events
function Status:click(event, up) end

function Status:scroll(event, step) end

function Status:touch(event, step) end

-- Children
function Status:children_add(fn, order, side)
	self._inc = self._inc + 1
	local children = side == self.RIGHT and self._right or self._left

	children[#children + 1] = { fn, id = self._inc, order = order }
	table.sort(children, function(a, b) return a.order < b.order end)

	return self._inc
end

function Status:children_remove(id, side)
	local children = side == self.RIGHT and self._right or self._left
	for i, child in ipairs(children) do
		if child.id == id then
			table.remove(children, i)
			break
		end
	end
end

function Status:children_redraw(side)
	local lines = {}
	for _, c in ipairs(side == self.RIGHT and self._right or self._left) do
		lines[#lines + 1] = (type(c[1]) == "string" and self[c[1]] or c[1])(self)
	end
	return ui.Line(lines)
end
