Status = {
	area = ui.Rect.default,
}

function Status.style()
	if cx.active.mode.is_select then
		return THEME.status.mode_select
	elseif cx.active.mode.is_unset then
		return THEME.status.mode_unset
	else
		return THEME.status.mode_normal
	end
end

function Status:mode()
	local mode = tostring(cx.active.mode):upper()
	if mode == "UNSET" then
		mode = "UN-SET"
	end

	local style = self.style()
	return ui.Line {
		ui.Span(THEME.status.separator_open):fg(style.bg),
		ui.Span(" " .. mode .. " "):style(style),
	}
end

function Status:size()
	local h = cx.active.current.hovered
	if h == nil then
		return ui.Line {}
	end

	local style = self.style()
	return ui.Line {
		ui.Span(" " .. ya.readable_size(h:size() or h.cha.length) .. " "):fg(style.bg):bg(THEME.status.separator_style.bg),
		ui.Span(THEME.status.separator_close):fg(THEME.status.separator_style.fg),
	}
end

function Status:name()
	local h = cx.active.current.hovered
	if h == nil then
		return ui.Span("")
	end

	return ui.Span(" " .. h.name)
end

function Status:permissions()
	local h = cx.active.current.hovered
	if h == nil then
		return ui.Line {}
	end

	local perm = h.cha:permissions()
	if perm == nil then
		return ui.Line {}
	end

	local spans = {}
	for i = 1, #perm do
		local c = perm:sub(i, i)
		local style = THEME.status.permissions_t
		if c == "-" then
			style = THEME.status.permissions_s
		elseif c == "r" then
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
	local cursor = cx.active.current.cursor
	local length = #cx.active.current.files
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
		ui.Span(" " .. THEME.status.separator_open):fg(THEME.status.separator_style.fg),
		ui.Span(percent):fg(style.bg):bg(THEME.status.separator_style.bg),
	}
end

function Status:position()
	local cursor = cx.active.current.cursor
	local length = #cx.active.current.files

	local style = self.style()
	return ui.Line {
		ui.Span(string.format(" %2d/%-2d ", cursor + 1, length)):style(style),
		ui.Span(THEME.status.separator_close):fg(style.bg),
	}
end

function Status:progress(area, offset)
	local progress = cx.tasks.progress
	local left = progress.total - progress.succ
	if left == 0 then
		return {}
	end

	local gauge = ui.Gauge(ui.Rect {
		x = math.max(0, area.w - offset - 21),
		y = area.y,
		w = math.max(0, math.min(20, area.w - offset - 1)),
		h = 1,
	})

	if progress.fail == 0 then
		gauge = gauge:gauge_style(THEME.status.progress_normal)
	else
		gauge = gauge:gauge_style(THEME.status.progress_error)
	end

	local percent = 99
	if progress.found ~= 0 then
		percent = math.min(99, progress.processed * 100 / progress.found)
	end

	return {
		gauge
			:percent(percent)
			:label(ui.Span(string.format("%3d%%, %d left", percent, left)):style(THEME.status.progress_label)),
	}
end

function Status:render(area)
	self.area = area

	local left = ui.Line { self:mode(), self:size(), self:name() }
	local right = ui.Line { self:permissions(), self:percentage(), self:position() }
	local progress = self:progress(area, right:width())
	return {
		ui.Paragraph(area, { left }),
		ui.Paragraph(area, { right }):align(ui.Paragraph.RIGHT),
		table.unpack(progress),
	}
end
