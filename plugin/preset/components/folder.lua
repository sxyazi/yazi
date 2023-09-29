Folder = {}

function Folder:parent(area)
	local parent = cx.manager.parent
	if parent == nil then
		return ui.Paragraph(area, ui.Line {})
	end

	local hovered = nil
	if parent.hovered ~= nil then
		hovered = parent.hovered.url
	end

	local lines = {}
	for _, f in pairs(parent.files) do
		local line = ui.Line { ui.Span(" " .. f.icon .. " " .. f.name .. " ") }

		-- TODO: preview hovered
		if f.url == hovered then
			line = line:style(THEME.selection.hovered)
		else
			line = line:style(f.style)
		end

		lines[#lines + 1] = line
	end

	return { ui.Paragraph(area, lines) }
end

function Folder:current(area)
	local hovered = nil
	if cx.manager.current.hovered ~= nil then
		hovered = cx.manager.current.hovered.url
	end

	local lines = {}
	for _, f in pairs(cx.manager.current.files) do
		local line = ui.Line { ui.Span(" " .. f.icon .. " " .. f.name .. " ") }

		-- TODO: preview hovered
		if f.url == hovered then
			line = line:style(THEME.selection.hovered)
		else
			line = line:style(f.style)
		end

		lines[#lines + 1] = line
	end

	return { ui.Paragraph(area, lines) }
end

function Folder:preview(area)
	local target = cx.manager.preview.folder
	if target == nil then
		return ui.Paragraph(area, ui.Line {})
	end

	local lines = {}
	for _, f in pairs(target.files) do
		lines[#lines + 1] = ui.Line { ui.Span(f.name) }
	end

	return { ui.Paragraph(area, lines) }
end

function Folder:render(area, args)
	if args.kind == 0 then
		return self:parent(area)
	elseif args.kind == 1 then
		return self:current(area)
	elseif args.kind == 2 then
		return self:preview(area)
	end
end
