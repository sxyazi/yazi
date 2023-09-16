function mode()
	local mode = cx.manager.mode:upper()
	if mode == "UNSET" then
		mode = "UN-SET"
	end

	return yazi
		.Line(
			yazi.Span(THEME.status.separator.opening):fg(THEME.status.mode_normal.bg),
			yazi.Span(" " .. mode .. " "):style(THEME.status.mode_normal)
		)
		:to_string()
end

function size()
	local hovered = cx.manager.hovered
	if hovered == nil then
		return ""
	end

	return yazi
		.Line(
			yazi.Span(" " .. hovered.length .. " "):fg(THEME.status.mode_normal.bg):bg(THEME.status.fancy.bg),
			yazi.Span(THEME.status.separator.closing):fg(THEME.status.fancy.bg)
		)
		:to_string()
end

function name()
	local hovered = cx.manager.hovered
	if hovered == nil then
		return ""
	end

	return yazi.Span(" " .. yazi.basename(hovered.url)):to_string()
end

function permissions() end

function percentage() end

function position() end
