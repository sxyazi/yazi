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
	local progress = cx.tasks.progress
	if progress.total == 0 then
		return {}
	end

	local gauge = ui.Gauge():area(self._area)
	if progress.fail == 0 then
		gauge = gauge:gauge_style(th.status.progress_normal)
	else
		gauge = gauge:gauge_style(th.status.progress_error)
	end

	local percent = 99
	if progress.found ~= 0 then
		percent = math.min(99, ya.round(progress.processed * 100 / progress.found))
	end

	local left = progress.total - progress.succ
	return gauge
		:percent(percent)
		:label(ui.Span(string.format("%3d%%, %d left", percent, left)):style(th.status.progress_label))
end
