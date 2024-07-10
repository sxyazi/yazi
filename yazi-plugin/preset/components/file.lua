File = {}

function File:icon(file)
	local icon = file:icon()
	if not icon then
		return {}
	elseif file:is_hovered() then
		return { ui.Span(" " .. icon.text .. " ") }
	else
		return { ui.Span(" " .. icon.text .. " "):style(icon.style) }
	end
end

function File:prefix(file)
	local prefix = file:prefix() or ""
	return prefix == "" and {} or { ui.Span(prefix .. "/") }
end

function File:highlights(file)
	local name = file.name:gsub("\r", "?", 1)
	local highlights = file:highlights()
	if not highlights or #highlights == 0 then
		return { ui.Span(name) }
	end

	local spans, last = {}, 0
	for _, h in ipairs(highlights) do
		if h[1] > last then
			spans[#spans + 1] = ui.Span(name:sub(last + 1, h[1]))
		end
		spans[#spans + 1] = ui.Span(name:sub(h[1] + 1, h[2])):style(THEME.manager.find_keyword)
		last = h[2]
	end
	if last < #name then
		spans[#spans + 1] = ui.Span(name:sub(last + 1))
	end
	return spans
end

function File:found(file)
	if not file:is_hovered() then
		return {}
	end

	local found = file:found()
	if not found then
		return {}
	end

	return {
		ui.Span("  "),
		ui.Span(string.format("[%d/%d]", found[1] + 1, found[2])):style(THEME.manager.find_position),
	}
end

function File:symlink(file)
	if not MANAGER.show_symlink then
		return {}
	end

	local to = file.link_to
	return to and { ui.Span(" -> " .. tostring(to)):italic() } or {}
end

function File:full(file)
	return ya.flat {
		self:icon(file),
		self:prefix(file),
		self:highlights(file),
		self:found(file),
		self:symlink(file),
	}
end

function File:style(file)
	local style = file:style()
	if not file:is_hovered() then
		return style
	elseif file:in_preview() then
		return style and style:patch(THEME.manager.preview_hovered) or THEME.manager.preview_hovered
	else
		return style and style:patch(THEME.manager.hovered) or THEME.manager.hovered
	end
end

function File:marker(file)
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
