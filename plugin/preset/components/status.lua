function layout(area)
	local chunks = yazi
		.Layout()
		:direction(false)
		:constraints({ yazi.Constraint.Percentage(50), yazi.Constraint.Percentage(50) })
		:split(area)

	return yazi.Paragraph.render(
		yazi.Paragraph(mode(), size(), name()):area(chunks[1]),
		yazi.Paragraph(permissions(), percentage(), position()):area(chunks[2])
	)
end

function mode()
	local mode = cx.manager.mode:upper()
	if mode == "UNSET" then
		mode = "UN-SET"
	end

	return yazi.Line(
		yazi.Span(THEME.status.separator.opening):fg(THEME.status.mode_normal.bg),
		yazi.Span(" " .. mode .. " "):style(THEME.status.mode_normal)
	)
end

function size()
	local hovered = cx.manager.current_hovered
	if hovered == nil then
		return yazi.Span("")
	end

	return yazi.Line(
		yazi.Span(" " .. hovered.length .. " "):fg(THEME.status.mode_normal.bg):bg(THEME.status.fancy.bg),
		yazi.Span(THEME.status.separator.closing):fg(THEME.status.fancy.bg)
	)
end

function name()
	local hovered = cx.manager.current_hovered
	if hovered == nil then
		return yazi.Span("")
	end

	return yazi.Span(" " .. utils.basename(hovered.url))
end

function permissions()
	local hovered = cx.manager.current_hovered
	if hovered == nil then
		return yazi.Span("")
	end

	if hovered.permissions == nil then
		return yazi.Span("")
	end

	local spans = {}
	for i = 1, #hovered.permissions do
		local c = hovered.permissions:sub(i, i)
		local style = THEME.status.permissions_t
		if c == "r" then
			style = THEME.status.permissions_r
		elseif c == "w" then
			style = THEME.status.permissions_w
		elseif c == "x" or c == "s" or c == "S" or c == "t" or c == "T" then
			style = THEME.status.permissions_x
		end
		spans[i] = yazi.Span(c):style(style)
	end
	return yazi.Line:from(spans)
end

function percentage()
	local percent = 0
	local cursor = cx.manager.current_cursor
	local length = cx.manager.current_length
	if cursor ~= 0 and length ~= 0 then
		percent = math.floor((cursor + 1) * 100 / length)
	end

	if percent == 0 then
		percent = "  Top "
	else
		percent = string.format(" %3d%% ", percent)
	end

	return yazi.Line(
		yazi.Span(THEME.status.separator.opening):fg(THEME.status.fancy.bg),
		yazi.Span(percent):fg(THEME.status.mode_normal.bg):bg(THEME.status.fancy.bg)
	)
end

function position()
	local cursor = cx.manager.current_cursor
	local length = cx.manager.current_length
	return yazi.Span(string.format(" %d/%d ", cursor + 1, length))
end
