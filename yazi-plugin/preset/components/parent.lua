Parent = {
	_id = "parent",
}

function Parent:new(area, tab)
	return setmetatable({
		_area = area,
		_tab = tab,
		_folder = tab.parent,
	}, { __index = self })
end

function Parent:reflow() return { self } end

function Parent:redraw()
	if not self._folder then
		return {}
	end

	local left, right = {}, {}
	for _, f in ipairs(self._folder.window) do
		local entity = Entity:new(f)
		left[#left + 1], right[#right + 1] = entity:redraw(), Linemode:new(f):redraw()

		local max = math.max(0, self._area.w - right[#right]:width())
		left[#left]:truncate { max = max, ellipsis = entity:ellipsis(max) }
	end

	return {
		ui.List(left):area(self._area),
		ui.Text(right):area(self._area):align(ui.Align.RIGHT),
	}
end

-- Mouse events
function Parent:click(event, up)
	local y = event.y - self._area.y + 1
	local window = self._folder and self._folder.window or {}
	if window[y] then
		Entity:new(window[y]):click(event, up)
	elseif not up and event.is_left then
		ya.emit("leave", {})
	end
end

function Parent:scroll(event, step) end

function Parent:touch(event, step) end
