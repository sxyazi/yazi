Marker = {
	_id = "marker",
}

function Marker:new(area, folder)
	return setmetatable({
		_area = area,
		_folder = folder,
	}, { __index = self })
end

function Marker:redraw()
	if self._area.w * self._area.h == 0 then
		return {}
	elseif not self._folder or #self._folder.window == 0 then
		return {}
	end

	local elements = {}
	local append = function(last)
		if not last[3] then
			return
		end

		local y = math.min(self._area.y + last[1], self._area.y + self._area.h) - 1
		local rect = ui.Rect {
			x = math.max(0, self._area.x - 1),
			y = y,
			w = 1,
			h = math.min(1 + last[2] - last[1], self._area.y + self._area.h - y),
		}
		elements[#elements + 1] = ui.Bar(ui.Edge.LEFT):area(rect):style(last[3])
	end

	local last = { 0, 0, nil } -- start, end, style
	for i, f in ipairs(self._folder.window) do
		local style = self:style(f)
		if i - last[2] > 1 or last[3] ~= style then
			append(last)
			last = { i, i, style }
		else
			last[2] = i
		end
	end

	append(last)
	return elements
end

function Marker:style(file)
	local marked = file:is_marked()
	if marked == 1 then
		return th.mgr.marker_marked
	elseif marked == 0 and file:is_selected() then
		return th.mgr.marker_selected
	end

	local yanked = file:is_yanked()
	if yanked == 1 then
		return th.mgr.marker_copied
	elseif yanked == 2 then
		return th.mgr.marker_cut
	end
end

-- Mouse events
function Marker:click(event, up) end

function Marker:scroll(event, step) end

function Marker:touch(event, step) end
