Header = {
	area = ui.Rect.default,
}

function Header:cwd(area)
	local cwd = cx.active.current.cwd

	local path
	if not cwd.is_search then
		path = ya.readable_path(tostring(cwd))
	else
		path = string.format("%s (search: %s)", ya.readable_path(tostring(cwd)), cwd:frag())
	end

	local width = area.right
	if #path > width then
		path = string.sub(path, #path - width + 1)
	end
	return ui.Span(path):style(THEME.manager.cwd)
end

function Header:count()
	local yanked = #cx.yanked

	local count, style
	if yanked == 0 then
		count = #cx.active.selected
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
			text = ya.truncate(text .. " " .. cx.tabs[i]:name(), THEME.manager.tab_width)
		end
		if i == cx.tabs.idx then
			spans[#spans + 1] = ui.Span(" " .. text .. " "):style(THEME.manager.tab_active)
		else
			spans[#spans + 1] = ui.Span(" " .. text .. " "):style(THEME.manager.tab_inactive)
		end
	end
	return ui.Line(spans)
end

function Header:layout(area)
	self.area = area

	return ui.Layout()
		:direction(ui.Layout.HORIZONTAL)
		:constraints({ ui.Constraint.Percentage(50), ui.Constraint.Percentage(50) })
		:split(area)
end

function Header:render(area)
	local chunks = self:layout(area)

	local left = ui.Line { self:cwd(chunks[1]) }
	local right = ui.Line { self:count(), self:tabs() }
	return {
		ui.Paragraph(chunks[1], { left }),
		ui.Paragraph(chunks[2], { right }):align(ui.Paragraph.RIGHT),
	}
end
