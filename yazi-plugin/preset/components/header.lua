Header = {
	area = ui.Rect.default,
}

function Header:cwd()
	local cwd = cx.active.current.cwd

	local span
	if not cwd.is_search then
		span = ui.Span(ya.readable_path(tostring(cwd)))
	else
		span = ui.Span(string.format("%s (search: %s)", ya.readable_path(tostring(cwd)), cwd:frag()))
	end
	return span:style(THEME.manager.cwd)
end

function Header:counter()
	local selected = #cx.active.selected
	local yanked = #cx.yanked

	local count
	local style
	if cx.yanked.is_cut then
		count = yanked
		style = THEME.manager.count_cut
	elseif yanked > 0 then
		count = yanked
		style = THEME.manager.count_copied
	else
		count = selected
		style = THEME.manager.count_selected
	end

	if count == 0 then
		return ui.Line({})
	else
		return ui.Line({
			ui.Span(string.format(" %s ", count)):style(style),
			ui.Span(" "),
		})
	end
end

function Header:tabs()
	local spans = {}
	for i = 1, #cx.tabs do
		local text = i
		if THEME.manager.tab_width > 2 then
			text = ya.truncate(text .. " " .. cx.tabs[i]:name(), THEME.manager.tab_width)
		end
		if i == cx.tabs.idx + 1 then
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

	local left = ui.Line { self:cwd() }
	local right = ui.Line { self:counter(), self:tabs() }
	return {
		ui.Paragraph(chunks[1], { left }),
		ui.Paragraph(chunks[2], { right }):align(ui.Paragraph.RIGHT),
	}
end
