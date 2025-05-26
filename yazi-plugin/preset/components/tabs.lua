Tabs = {
	_id = "tabs",
	_offsets = {},
}

function Tabs:new(area)
	return setmetatable({
		_area = area,
	}, { __index = self })
end

function Tabs:reflow() return { self } end

function Tabs:redraw()
	if self.height() < 1 then
		return {}
	end

	local lines = {
		ui.Line(th.tabs.sep_outer.open):fg(th.tabs.inactive.bg),
	}

	local pos = lines[1]:width()
	local max = math.floor(self:inner_width() / #cx.tabs)
	for i = 1, #cx.tabs do
		local name = ya.truncate(string.format(" %d %s ", i, cx.tabs[i].name), { max = max })
		if i == cx.tabs.idx then
			lines[#lines + 1] = ui.Line {
				ui.Span(th.tabs.sep_inner.open):style(th.tabs.inactive),
				ui.Span(name):style(th.tabs.active),
				ui.Span(th.tabs.sep_inner.close):style(th.tabs.inactive),
			}
		else
			lines[#lines + 1] = ui.Line(name):style(th.tabs.inactive)
		end
		self._offsets[i], pos = pos, pos + lines[#lines]:width()
	end

	lines[#lines + 1] = ui.Line(th.tabs.sep_outer.close):fg(th.tabs.inactive.bg)
	return ui.Line(lines):area(self._area)
end

function Tabs.height() return #cx.tabs > 1 and 1 or 0 end

function Tabs:inner_width()
	local si, so = th.tabs.sep_inner, th.tabs.sep_outer
	return math.max(0, self._area.w - ui.Line({ si.open, si.close, so.open, so.close }):width())
end

-- Mouse events
function Tabs:click(event, up)
	if up or event.is_middle then
		return
	end
	for i = #self._offsets, 1, -1 do
		if event.x >= self._offsets[i] then
			ya.emit("tab_switch", { i - 1 })
			break
		end
	end
end

function Tabs:scroll(event, step) end

function Tabs:touch(event, step) end
