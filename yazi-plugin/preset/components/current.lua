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
	local line
	if self._folder.files.filter then
		line = ui.Line("No filter results")
	else
		line = ui.Line(self._folder.stage == "loading" and "Loading..." or "No items")
	end

	return {
		ui.Paragraph(self._area, { line }):align(ui.Paragraph.CENTER),
	}
end

function Current:render()
	local files = self._folder.window
	if #files == 0 then
		return self:empty()
	end

	local entities, linemodes = {}, {}
	for _, f in ipairs(files) do
		linemodes[#linemodes + 1] = Linemode:new(f):render()

		local entity = Entity:new(f)
		entities[#entities + 1] = ui.ListItem(entity:render()):style(entity:style())
	end

	return {
		ui.List(self._area, entities),
		ui.Paragraph(self._area, linemodes):align(ui.Paragraph.RIGHT),
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

	ya.manager_emit("arrow", { y + f.offset - f.hovered.idx })
	if event.is_right then
		ya.manager_emit("open", {})
	end
end

function Current:scroll(event, step) ya.manager_emit("arrow", { step }) end

function Current:touch(event, step) end
