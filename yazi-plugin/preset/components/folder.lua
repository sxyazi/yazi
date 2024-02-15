Folder = {
	PARENT = 0,
	CURRENT = 1,
	PREVIEW = 2,
}

function Folder:by_kind(kind)
	if kind == self.PARENT then
		return cx.active.parent
	elseif kind == self.CURRENT then
		return cx.active.current
	elseif kind == self.PREVIEW then
		return cx.active.preview.folder
	end
end

function Folder:icon(file)
	local icon = file:icon()
	return icon and ui.Span(" " .. icon.text .. " "):style(icon.style) or ui.Span("")
end

function Folder:highlight_ranges(s, ranges)
	if not ranges or #ranges == 0 then
		return { ui.Span(s) }
	end

	local spans = {}
	local last = 0
	for _, r in ipairs(ranges) do
		if r[1] > last then
			spans[#spans + 1] = ui.Span(s:sub(last + 1, r[1]))
		end
		spans[#spans + 1] = ui.Span(s:sub(r[1] + 1, r[2])):style(THEME.manager.find_keyword)
		last = r[2]
	end
	if last < #s then
		spans[#spans + 1] = ui.Span(s:sub(last + 1))
	end
	return spans
end

function Folder:highlighted_name(file)
	-- Complete prefix when searching across directories
	local prefix = file:prefix() or ""
	if prefix ~= "" then
		prefix = prefix .. "/"
	end

	-- Range highlighting for filenames
	local highlights = file:highlights()
	local spans = self:highlight_ranges(prefix .. file.name, highlights)

	-- Show symlink target
	if MANAGER.show_symlink and file.link_to ~= nil then
		spans[#spans + 1] = ui.Span(" -> " .. tostring(file.link_to)):italic()
	end

	if not highlights or not file:is_hovered() then
		return spans
	end

	local found = file:found()
	if found ~= nil then
		spans[#spans + 1] = ui.Span("  ")
		spans[#spans + 1] = ui.Span(string.format("[%d/%d]", found[1] + 1, found[2])):style(THEME.manager.find_position)
	end
	return spans
end

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
		end

		spans[#spans + 1] = ui.Span(" ")
		lines[#lines + 1] = ui.Line(spans)
	end
	return ui.Paragraph(area, lines):align(ui.Paragraph.RIGHT)
end

function Folder:marker(file)
	local yanked = file:is_yanked()
	if yanked ~= 0 then
		return yanked -- 1: copied, 2: cut
	end

	local marked = file:is_marked()
	if marked == 1 then
		return 3 -- 3: marked
	elseif marked == 0 and file:is_selected() then
		return 4 -- 4: selected
	end
	return 0
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
