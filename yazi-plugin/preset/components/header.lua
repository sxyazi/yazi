Header = {
	area = ui.Rect.default,
}

function Header:cwd(max)
	local cwd = cx.active.current.cwd
	local readable = ya.readable_path(tostring(cwd))

	local text = cwd.is_search and string.format("%s (search: %s)", readable, cwd:frag()) or readable
	return ui.Span(ya.truncate(text, { max = max, rtl = true })):style(THEME.manager.cwd)
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

-- TODO: remove this function after v0.2.5 release
function Header:layout(area)
	if not ya.deprecated_header_layout then
		ya.deprecated_header_layout = true
		ya.notify {
			title = "Deprecated API",
			content = "`Header:layout()` is deprecated, please apply the latest `Header:render()` in your `init.lua`",
			timeout = 5,
			level = "warn",
		}
	end

	self.area = area

	return ui.Layout()
		:direction(ui.Layout.HORIZONTAL)
		:constraints({ ui.Constraint.Percentage(50), ui.Constraint.Percentage(50) })
		:split(area)
end

function Header:render(area)
	self.area = area

	local right = ui.Line { self:count(), self:tabs() }
	local left = ui.Line { self:cwd(math.max(0, area.w - right:width())) }
	return {
		ui.Paragraph(area, { left }),
		ui.Paragraph(area, { right }):align(ui.Paragraph.RIGHT),
	}
end
