Status = {}

function Status.style()
	local mode = cx.manager.mode:upper()
	if mode == "SELECT" then
		return THEME.status.mode_select
	elseif mode == "UNSET" then
		return THEME.status.mode_unset
	else
		return THEME.status.mode_normal
	end
end

function Status:mode()
	local mode = cx.manager.mode:upper()
	if mode == "UNSET" then
		mode = "UN-SET"
	end

	local style = self.style()
	return ui.Line {
		ui.Span(THEME.status.separator.opening):fg(style.bg),
		ui.Span(" " .. mode .. " "):style(style),
	}
end

function Status:size()
	local h = cx.manager.current.hovered
	if h == nil then
		return ui.Span("")
	end

	local style = self.style()
	return ui.Line {
		ui.Span(" " .. utils.readable_size(h.length) .. " "):fg(style.bg):bg(THEME.status.fancy.bg),
		ui.Span(THEME.status.separator.closing):fg(THEME.status.fancy.bg),
	}
end

function Status:name()
	local h = cx.manager.current.hovered
	if h == nil then
		return ui.Span("")
	end

	return ui.Span(" " .. h.name)
end

function Status:permissions()
	local h = cx.manager.current.hovered
	if h == nil or h.permissions == nil then
		return ui.Span("")
	end

	local spans = {}
	for i = 1, #h.permissions do
		local c = h.permissions:sub(i, i)
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
	return ui.Line(spans)
end

function Status:percentage()
	local percent = 0
	local cursor = cx.manager.current.cursor
	local length = #cx.manager.current.files
	if cursor ~= 0 and length ~= 0 then
		percent = math.floor((cursor + 1) * 100 / length)
	end

	if percent == 0 then
		percent = "  Top "
	else
		percent = string.format(" %3d%% ", percent)
	end

	local style = self.style()
	return ui.Line {
		ui.Span(" " .. THEME.status.separator.opening):fg(THEME.status.fancy.bg),
		ui.Span(percent):fg(style.bg):bg(THEME.status.fancy.bg),
	}
end

function Status:position()
	local cursor = cx.manager.current.cursor
	local length = #cx.manager.current.files

	local style = self.style()
	return ui.Line {
		ui.Span(string.format(" %2d/%-2d ", cursor + 1, length)):style(style),
		ui.Span(THEME.status.separator.closing):fg(style.bg),
	}
end

function Status:render(area)
	local chunks = ui.Layout()
		:direction(ui.Direction.HORIZONTAL)
		:constraints({ ui.Constraint.Percentage(50), ui.Constraint.Percentage(50) })
		:split(area)

	local left = ui.Line { self:mode(), self:size(), self:name() }
	local right = ui.Line { self:permissions(), self:percentage(), self:position() }
	return { ui.Paragraph(chunks[1], { left }), ui.Paragraph(chunks[2], { right }):align(ui.Alignment.RIGHT) }
end
