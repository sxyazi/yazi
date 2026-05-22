Current = {
	_id = "current",
}

local last_click = { time = 0, url = nil }

function Current:new(area, tab)
	return setmetatable({
		_area = area,
		_tab = tab,
		_folder = tab.current,
	}, { __index = self })
end

function Current:empty()
	local s
	if self._folder.files.filter then
		s = "No filter results"
	else
		local done, err = self._folder.stage()
		s = not done and "Loading..." or not err and "No items" or string.format("Error: %s", err)
	end

	return {
		ui.Text(s):area(self._area):align(ui.Align.CENTER):wrap(ui.Wrap.YES),
	}
end

function Current:reflow() return { self } end

function Current:redraw()
	local files = self._folder.window
	if #files == 0 then
		return self:empty()
	end

	local left, right = {}, {}
	for _, f in ipairs(files) do
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
function Current:click(event, up)
	if up or event.is_middle then
		return
	end

	local y = event.y - self._area.y + 1
	local f = self._folder.window[y]
	if not f then
		return
	end

	if event.is_left then
		local delay = rt.mgr.mouse_double_click_delay or 0
		local now = ya.time()
		local url = tostring(f.url)
		if delay > 0
			and (now - last_click.time) * 1000 <= delay
			and last_click.url == url
		then
			last_click.time = 0
			ya.emit("open", {})
			return
		end
		last_click = { time = now, url = url }
	end

	Entity:new(f):click(event, up)
end

function Current:scroll(event, step) ya.emit("arrow", { step }) end

function Current:touch(event, step) end
