Progress = {
	_id = "progress",
}

function Progress:new(area, offset)
	local me = setmetatable({ _area = area, _offset = offset }, { __index = self })
	me:layout()
	return me
end

function Progress:use(area) return setmetatable({ _area = area }, { __index = self }) end

function Progress:layout()
	self._area = ui.Rect {
		x = math.max(0, self._area.w - self._offset - 21),
		y = self._area.y,
		w = ya.clamp(0, self._area.w - self._offset - 1, 20),
		h = math.min(1, self._area.h),
	}
end

function Progress:reflow() return { self } end

function Progress:redraw()
	local summary = cx.tasks.summary
	if summary.total == 0 then
		return {}
	end

	local gauge = ui.Gauge():area(self._area)
	if summary.failed == 0 then
		gauge:gauge_style(th.status.progress_normal)
	else
		gauge:gauge_style(th.status.progress_error)
	end

	local label, percent = "", summary.percent
	if percent then
		label = string.format("%3d%%, ", math.floor(percent))
	else
		percent = 0
	end

	label = label .. string.format("%d left", summary.total - summary.success)
	return {
		ui.Clear(self._area),
		gauge:percent(percent):label(ui.Span(label):style(th.status.progress_label)),
	}
end
