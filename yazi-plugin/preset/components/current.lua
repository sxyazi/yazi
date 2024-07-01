Current = {
	area = ui.Rect.default,
}

function Current:empty(area)
	local folder = Folder:by_kind(Folder.CURRENT)

	local line
	if folder.files.filter then
		line = ui.Line("No filter results")
	else
		line = ui.Line(folder.stage == "loading" and "Loading..." or "No items")
	end

	return {
		ui.Paragraph(area, { line }):align(ui.Paragraph.CENTER),
	}
end

function Current:render(area)
	self.area = area

	local files = Folder:by_kind(Folder.CURRENT).window
	if #files == 0 then
		return self:empty(area)
	end

	local items, markers = {}, {}
	for i, f in ipairs(files) do
		items[#items + 1] = ui.ListItem(ui.Line(File:full(f))):style(File:style(f))

		-- Yanked/marked/selected files
		local marker = File:marker(f)
		if marker ~= 0 then
			markers[#markers + 1] = { i, marker }
		end
	end

	return ya.flat {
		ui.List(area, items),
		Folder:linemode(area, files),
		Folder:markers(area, markers),
	}
end

function Current:click(event, up)
	if up or event.is_middle then
		return
	end

	local f = Folder:by_kind(Folder.CURRENT)
	if event.y > #f.window or not f.hovered then
		return
	end

	ya.manager_emit("arrow", { event.y + f.offset - f.hovered.idx })
	if event.is_right then
		ya.manager_emit("open", {})
	end
end

function Current:scroll(event, step) ya.manager_emit("arrow", { step }) end

function Current:touch(event, step) end
