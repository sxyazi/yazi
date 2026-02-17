Current = {
	_id = "current",
	_anchor = nil,
	_last_click = nil,
}

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
	if up or event.is_middle then return end

	local y = event.y - self._area.y + 1
	local file = self._folder.window[y]
	if not file then return end

	local abs = self._folder.offset + y

	-- Reset anchor and clear selection on directory change
	local cwd = tostring(self._folder.cwd)
	if Current._cwd ~= cwd then
		Current._cwd = cwd
		Current._anchor = nil
		ya.emit("escape", { select = true })
	end

	-- Double-click detection
	local last = Current._last_click
	if event.is_left and last and last.y == y and ya.time() - last.time < 0.4 then
		Current._last_click = nil
		ya.emit("reveal", { file.url })
		ya.emit("open", {})
		return
	end
	if event.is_left then
		Current._last_click = { y = y, time = ya.time() }
	end

	if event.is_left and (event.is_super or event.is_ctrl) then
		-- Cmd/Ctrl-click: toggle individual selection
		ya.emit("toggle", { file.url })
		ya.emit("reveal", { file.url })
		Current._anchor = abs
	elseif event.is_left and event.is_shift then
		-- Shift-click: range select from anchor
		local anchor = Current._anchor or (self._folder.cursor + 1)
		local lo = math.min(anchor, abs)
		local hi = math.max(anchor, abs)
		local urls = {}
		local files = self._folder.files
		for i = lo, hi do
			local f = files[i]
			if f then urls[#urls + 1] = f.url end
		end
		urls.state = "on"
		ya.emit("toggle_all", urls)
		ya.emit("reveal", { file.url })
		Current._anchor = abs
	elseif event.is_right then
		-- Right-click: open interactive ("Open with...")
		ya.emit("reveal", { file.url })
		ya.emit("open", { interactive = true })
		Current._anchor = abs
	else
		-- Plain left-click: reveal + set anchor
		ya.emit("reveal", { file.url })
		Current._anchor = abs
	end
end

function Current:scroll(event, step) ya.emit("arrow", { step }) end

function Current:touch(event, step) end
