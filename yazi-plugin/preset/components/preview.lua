Preview = {
	_id = "preview",
}

function Preview:new(area, tab)
	return setmetatable({
		_area = area,
		_tab = tab,
		_folder = tab.preview.folder,
	}, { __index = self })
end

function Preview:reflow() return { self } end

function Preview:redraw() return {} end

-- Mouse events
function Preview:click(event, up)
	if up or not event.is_left then
		return
	end

	local y = event.y - self._area.y + 1
	local window = self._folder and self._folder.window or {}
	if window[y] then
		ya.emit("reveal", { window[y].url })
	else
		ya.emit("enter", {})
	end
end

function Preview:scroll(event, step) ya.emit("seek", { step }) end

function Preview:touch(event, step) end
