Folder = {
	PARENT = 0,
	CURRENT = 1,
	PREVIEW = 2,
}

function Folder:linemode(area, files)
	local mode = cx.active.conf.linemode
	if mode == "none" then
		return {}
	end

	local lines = {}
	for _, f in ipairs(files) do
		local spans = { ui.Span(" ") }
		if mode == "size" then
			local size = f:size()
			spans[#spans + 1] = ui.Span(size and ya.readable_size(size) or "")
		elseif mode == "mtime" then
			local time = f.cha.modified
			spans[#spans + 1] = ui.Span(time and os.date("%y-%m-%d %H:%M", time // 1) or "")
		elseif mode == "permissions" then
			spans[#spans + 1] = ui.Span(f.cha:permissions() or "")
		elseif mode == "owner" then
			spans[#spans + 1] = ya.user_name and ui.Span(ya.user_name(f.cha.uid) .. ":" .. ya.group_name(f.cha.gid))
				or ui.Span("")
		end

		spans[#spans + 1] = ui.Span(" ")
		lines[#lines + 1] = ui.Line(spans)
	end
	return ui.Paragraph(area, lines):align(ui.Paragraph.RIGHT)
end

function Folder:markers(area, markers)
	if #markers == 0 or area.w * area.h == 0 then
		return {}
	end

	local elements = {}
	local append = function(last)
		local y = math.min(area.y + last[1], area.y + area.h) - 1
		local bar = ui.Bar(
			ui.Rect {
				x = math.max(0, area.x - 1),
				y = y,
				w = 1,
				h = math.min(1 + last[2] - last[1], area.y + area.h - y),
			},
			ui.Bar.LEFT
		)

		if last[3] == 1 then
			bar = bar:style(THEME.manager.marker_copied)
		elseif last[3] == 2 then
			bar = bar:style(THEME.manager.marker_cut)
		elseif last[3] == 3 then
			bar = bar:style(THEME.manager.marker_marked)
		elseif last[3] == 4 then
			bar = bar:style(THEME.manager.marker_selected)
		end
		elements[#elements + 1] = bar
	end

	local last = { markers[1][1], markers[1][1], markers[1][2] } -- start, end, type
	for _, m in ipairs(markers) do
		if m[1] - last[2] > 1 or last[3] ~= m[2] then
			append(last)
			last = { m[1], m[1], m[2] }
		else
			last[2] = m[1]
		end
	end

	append(last)
	return elements
end

function Folder:by_kind(kind)
	if kind == self.PARENT then
		return cx.active.parent
	elseif kind == self.CURRENT then
		return cx.active.current
	elseif kind == self.PREVIEW then
		return cx.active.preview.folder
	end
end

function Folder:window(kind)
	local folder = self:by_kind(kind)
	return folder and folder.window
end
