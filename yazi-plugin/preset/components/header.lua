Header = {
	_id = "header",
}

function Header:new(area, tab)
	return setmetatable({
		_area = area,
		_tab = tab,
	}, { __index = self })
end

function Header:cwd(max)
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
	local right = ui.Line { self:count(), self:tabs() }
	local left = ui.Line { self:cwd(math.max(0, self._area.w - right:width())) }
	return {
		ui.Paragraph(self._area, { left }),
		ui.Paragraph(self._area, { right }):align(ui.Paragraph.RIGHT),
	}
end

-- Mouse events
function Header:click(event, up) end

function Header:scroll(event, step) end

function Header:touch(event, step) end
