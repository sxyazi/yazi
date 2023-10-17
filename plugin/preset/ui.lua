ui = {
	Alignment = {
		LEFT = 0,
		CENTER = 1,
		RIGHT = 2,
	},
	Direction = {
		HORIZONTAL = false,
		VERTICAL = true,
	},
	Position = {
		NONE = 0,
		TOP = 1,
		RIGHT = 2,
		BOTTOM = 3,
		LEFT = 4,
	},

	Base = setmetatable({
		PREVIEW = 0,
	}, {
		__call = function(_, ...) return ui.Base.new(...) end,
	}),
	Padding = setmetatable({
		left = function(left) return ui.Padding.new(left, 0, 0, 0) end,
		right = function(right) return ui.Padding.new(0, right, 0, 0) end,
		top = function(top) return ui.Padding.new(0, 0, top, 0) end,
		bottom = function(bottom) return ui.Padding.new(0, 0, 0, bottom) end,
		x = function(x) return ui.Padding.new(x, x, 0, 0) end,
		y = function(y) return ui.Padding.new(0, 0, y, y) end,
	}, {
		__call = function(_, ...) return ui.Padding.new(...) end,
	}),
}

function ui.highlight_ranges(s, ranges)
	if ranges == nil or #ranges == 0 then
		return { ui.Span(s) }
	end

	local spans = {}
	local last = 0
	for _, r in ipairs(ranges) do
		if r[1] > last then
			spans[#spans + 1] = ui.Span(s:sub(last + 1, r[1]))
		end
		-- TODO: use a customable style
		spans[#spans + 1] = ui.Span(s:sub(r[1] + 1, r[2])):style(THEME.manager.find_keyword)
		last = r[2]
	end
	if last < #s then
		spans[#spans + 1] = ui.Span(s:sub(last + 1))
	end
	return spans
end
