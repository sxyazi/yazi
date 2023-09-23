Status = {}

function Status.mode()
	local mode = cx.manager.mode:upper()
	if mode == "UNSET" then
		mode = "UN-SET"
	end

	return ui.Line(
		ui.Span(THEME.status.separator.opening):fg(THEME.status.mode_normal.bg),
		ui.Span(" " .. mode .. " "):style(THEME.status.mode_normal)
	)
end

function Status.size()
	local hovered = cx.manager.current_hovered
	if hovered == nil then
		return ui.Span("")
	end

	return ui.Line(
		ui.Span(" " .. hovered.length .. " "):fg(THEME.status.mode_normal.bg):bg(THEME.status.fancy.bg),
		ui.Span(THEME.status.separator.closing):fg(THEME.status.fancy.bg)
	)
end

function Status.name()
	local hovered = cx.manager.current_hovered
	if hovered == nil then
		return ui.Span("")
	end

	return ui.Span(" " .. utils.basename(hovered.url))
end

function Status.permissions()
	local hovered = cx.manager.current_hovered
	if hovered == nil then
		return ui.Span("")
	end

	if hovered.permissions == nil then
		return ui.Span("")
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
		spans[i] = ui.Span(c):style(style)
	end
	return ui.Line:from(spans)
end

function Status.percentage()
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

	return ui.Line(
		ui.Span(THEME.status.separator.opening):fg(THEME.status.fancy.bg),
		ui.Span(percent):fg(THEME.status.mode_normal.bg):bg(THEME.status.fancy.bg)
	)
end

function Status.position()
	local cursor = cx.manager.current_cursor
	local length = cx.manager.current_length
	return ui.Span(string.format(" %d/%d ", cursor + 1, length))
end

function Status:render(area)
	local chunks = ui.Layout()
		:direction(ui.Direction.HORIZONTAL)
		:constraints({ ui.Constraint.Percentage(50), ui.Constraint.Percentage(50) })
		:split(area)

	local left = ui.Line(self.mode(), self.size(), self.name())
	local right = ui.Line(self.permissions(), self.percentage(), self.position())
	return ui.Paragraph.render(
		ui.Paragraph(left):area(chunks[1]),
		ui.Paragraph(right):align(ui.Alignment.RIGHT):area(chunks[2])
	)
end
