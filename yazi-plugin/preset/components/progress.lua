Progress = {
	_area = ui.Rect.default, -- TODO: remove this
}

function Progress:redraw(area, offset)
	self._area = ui.Rect {
		x = math.max(0, area.w - offset - 21),
		y = area.y,
		w = ya.clamp(0, area.w - offset - 1, 20),
		h = math.min(1, area.h),
	}
	return self:partial_redraw()
end

-- Progress bars usually need frequent updates to report the latest task progress.
-- We use `partial_redraw()` to partially redraw it when there is progress change,
-- which has almost no cost compared to a full redraw by `redraw()`.
function Progress:partial_redraw()
	local progress = cx.tasks.progress
	if progress.total == 0 then
		return { ui.Text {} }
	end

	local gauge = ui.Gauge():area(self._area)
	if progress.fail == 0 then
		gauge = gauge:gauge_style(THEME.status.progress_normal)
	else
		gauge = gauge:gauge_style(THEME.status.progress_error)
	end

	local percent = 99
	if progress.found ~= 0 then
		percent = math.min(99, ya.round(progress.processed * 100 / progress.found))
	end

	local left = progress.total - progress.succ
	return {
		gauge
			:percent(percent)
			:label(ui.Span(string.format("%3d%%, %d left", percent, left)):style(THEME.status.progress_label)),
	}
end
