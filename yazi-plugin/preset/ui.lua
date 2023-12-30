ui = {}

function ui.highlight_ranges(s, ranges)
	if ranges == nil or #ranges == 0 then
		return { ui.Span(s) }
	end

	local spans = {}
	local last = 0
	for _, r in ipairs(ranges) do
		if r[1] > last then
			spans[#spans + 1] = ui.Span(s:sub(last + 1, r[1]))
		end
		spans[#spans + 1] = ui.Span(s:sub(r[1] + 1, r[2])):style(THEME.manager.find_keyword)
		last = r[2]
	end
	if last < #s then
		spans[#spans + 1] = ui.Span(s:sub(last + 1))
	end
	return spans
end
