Tasks = {
	_id = "tasks",
	_window = 2,
	_samples = {},
}

function Tasks:new(area)
	local me = setmetatable({ _area = area }, { __index = self })
	me:layout()
	return me
end

function Tasks:layout()
	self._area = self._area:pad(ui.Pad(1, 1, 1, 3))
	self._chunks = ui.Layout()
		:direction(ui.Layout.HORIZONTAL)
		:constraints({
			ui.Constraint.Percentage(60),
			ui.Constraint.Percentage(40),
		})
		:split(self._area)
end

function Tasks:reflow() return { self } end

function Tasks:redraw()
	self:sample()

	local elements = {}
	for i, snap in ipairs(cx.tasks.snaps) do
		local y = self._area.y + (i - 1) * 3
		if y >= self._area.bottom then
			break
		end

		elements[#elements + 1] = ui.Line({ self:icon(snap), snap.title }):area(self._area { y = y, h = 1 })

		if i == cx.tasks.cursor + 1 then
			elements[#elements] = elements[#elements]:style(th.tasks.hovered)
		end

		for _, e in ipairs(self:progress_redraw(snap, y + 1)) do
			elements[#elements + 1] = e
		end

		elements[#elements + 1] = ui.Bar(ui.Edge.LEFT)
			:area(ui.Rect {
				x = math.max(0, self._area.x - 2),
				y = y,
				w = self._area.w,
				h = 2,
			})
			:symbol("┃")

		if i == cx.tasks.cursor + 1 then
			elements[#elements] = elements[#elements]:style(th.tasks.hovered)
		end
	end

	return elements
end

function Tasks:sample()
	local now, samples = ya.time().unix, {}
	for _, snap in ipairs(cx.tasks.snaps) do
		local p = snap.prog
		if p.processed_bytes then
			local list = Tasks._samples[snap.title] or {}
			if #list == 0 or list[#list].bytes ~= p.processed_bytes or now - list[#list].at >= 1 then
				list[#list + 1] = { at = now, bytes = p.processed_bytes }
			end
			while #list > 2 and now - list[2].at > self._window do
				table.remove(list, 1)
			end
			samples[snap.title] = list
		end
	end
	Tasks._samples = samples
end

function Tasks:speed(snap)
	local list = Tasks._samples[snap.title]
	if not list or #list < 2 then
		return 0
	end

	local first, last = list[1], list[#list]
	return last.at > first.at and (last.bytes - first.bytes) / (last.at - first.at) or 0
end

function Tasks:eta(secs)
	secs = math.ceil(secs)
	if secs >= 3600 then
		return string.format("%dh%02dm", secs // 3600, secs % 3600 // 60)
	elseif secs >= 60 then
		return string.format("%dm%02ds", secs // 60, secs % 60)
	else
		return string.format("%ds", secs)
	end
end

function Tasks:icon(snap)
	if snap.prog.kind == "FileCopy" then
		return "  "
	elseif snap.prog.kind == "FileMove" then
		return "  "
	elseif snap.prog.kind == "FileDelete" then
		return "  "
	elseif snap.prog.kind == "FileDownload" then
		return "  "
	elseif snap.prog.kind == "FileUpload" then
		return "  "
	else
		return "  "
	end
end

function Tasks:progress(snap)
	local p = snap.prog

	local label, count, failed
	if p.total_bytes then
		local percent = snap.running and snap.cooked and "Cleaning…" or string.format("%3d%%", math.floor(snap.percent))
		label = string.format("%s - %s / %s", percent, ya.readable_size(p.processed_bytes), ya.readable_size(p.total_bytes))

		local speed = self:speed(snap)
		if speed > 0 then
			local left = math.max(0, p.total_bytes - p.processed_bytes) / speed
			label = string.format("%s - %s/s, %s left", label, ya.readable_size(speed), self:eta(left))
		end

		count = string.format("%d/%d", p.success_files, p.total_files)
		count = p.failed_files == 0 and count or string.format("%s, %d failed", count, p.failed_files)
		failed = snap.failed or p.failed_files > 0
	elseif p.kind == "Custom" and snap.percent then
		label = string.format("%3d%%", math.floor(snap.percent))
		count = string.format("%d/%d", p.success, p.total)
		count = p.failed == 0 and count or string.format("%s, %d failed", count, p.failed)
		failed = snap.failed or p.failed > 0
	end

	return label, count, failed
end

function Tasks:status(snap)
	if snap.running then
		return snap.cooked and "Cleaning…" or "Running…"
	else
		return "Failed, press Enter to view log…"
	end
end

function Tasks:progress_redraw(snap, y)
	local label, count, failed = self:progress(snap)
	if not label then
		return {
			ui.Line(self:status(snap)):fg("gray"):area(self._chunks[1] { y = y, h = 1 }),
		}
	end

	return {
		ui.Gauge()
			:area(self._chunks[1] { y = y, h = 1 })
			:percent(snap.percent)
			:label(ui.Span(label):style(th.status.progress_label))
			:gauge_style(failed and th.status.progress_error or th.status.progress_normal),
		ui.Line(count):fg("gray"):area(self._chunks[2] { y = y, h = 1 }):align(ui.Align.RIGHT),
	}
end
