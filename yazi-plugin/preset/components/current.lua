Current = {
	_id = "current",
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
		ui.Text(s):area(self._area):align(ui.Text.CENTER),
	}
end

function Current:reflow() return { self } end

function Current:redraw()
	local files = self._folder.window
	if #files == 0 then
		return self:empty()
	end

	local entities, linemodes = {}, {}
	for _, f in ipairs(files) do
		entities[#entities + 1] = Entity:new(f):redraw()
		linemodes[#linemodes + 1] = Linemode:new(f):redraw()
	end

	return {
		ui.List(entities):area(self._area),
		ui.Text(linemodes):area(self._area):align(ui.Text.RIGHT),
	}
end

-- Mouse events
function Current:click(event, up)
	if up or event.is_middle then
		return
	end

	local f = self._folder
	local y = event.y - self._area.y + 1
	if y > #f.window or not f.hovered then
		return
	end

	ya.mgr_emit("arrow", { y + f.offset - f.hovered.idx })
	if event.is_right then
		ya.mgr_emit("open", {})
	end
end

function Current:scroll(event, step) ya.mgr_emit("arrow", { step }) end

function Current:touch(event, step) end
