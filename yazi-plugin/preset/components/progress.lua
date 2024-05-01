---@class yazi.Progress
Progress = {
	area = ui.Rect.default,
}

---@param area unknown
---@param offset integer
---@return table
function Progress:render(area, offset)
	self.area = ui.Rect {
		x = math.max(0, area.w - offset - 21),
		y = area.y,
		w = ya.clamp(0, area.w - offset - 1, 20),
		h = math.min(1, area.h),
	}
	return self:partial_render()
end

-- Progress bars usually need frequent updates to report the latest task progress.
-- We use `partial_render()` to partially render it when there is progress change,
-- which has almost no cost compared to a full render by `render()`.
--
-- However, at this time, we can only access `cx.tasks`. If you need certain data from the complete `cx`,
-- just cache it to `self` during `render()`, and read it in `partial_render()` - this process is referred to as "composition".
---@return table
function Progress:partial_render()
	local progress = cx.tasks.progress
	if progress.total == 0 then
		return { ui.Paragraph(self.area, {}) }
	end

	local gauge = ui.Gauge(self.area)
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
