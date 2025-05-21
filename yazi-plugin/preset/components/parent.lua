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

	local items = {}
	for _, f in ipairs(self._folder.window) do
		local entity = Entity:new(f)
		items[#items + 1] = entity:redraw():truncate {
			max = self._area.w,
			ellipsis = entity:ellipsis(self._area.w),
		}
	end

	return ui.List(items):area(self._area)
end

-- Mouse events
function Parent:click(event, up)
	if up or not event.is_left then
		return
	end

	local y = event.y - self._area.y + 1
	local window = self._folder and self._folder.window or {}
	if window[y] then
		ya.emit("reveal", { window[y].url })
	else
		ya.emit("leave", {})
	end
end

function Parent:scroll(event, step) end

function Parent:touch(event, step) end
