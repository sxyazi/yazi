Rail = {}

function Rail:new(id, area, chunks)
	return setmetatable({
		_id = id,
		_area = area,
		_chunks = chunks,
	}, { __index = self })
end

function Rail:reflow() return { self } end

function Rail:redraw()
	return {
		ui.Bar(ui.Edge.LEFT):area(self._area):symbol(th.mgr.border_symbol):style(th.mgr.border_style),
	}
end

-- Mouse events
function Rail:click(event, up) end

function Rail:scroll(event, step) end

function Rail:touch(event, step) end

function Rail:drag(event)
	local c, x, parent, current, preview = self._chunks, 0, 0, 0, 0
	if self._id == "rail-left" then
		x = math.min(event.x, c[2].right - 2)
		parent = math.max(1, x - c[1].x)
		current = math.max(1, c[1].w + c[2].w - parent)
		preview = math.max(1, c[3].w)
	else
		x = math.max(event.x, c[2].x + 2)
		preview = math.max(1, c[3].right - x)
		current = math.max(1, c[2].w + c[3].w - preview)
		parent = math.max(1, c[1].w)
	end

	local r = rt.mgr.ratio
	if r.parent ~= parent or r.current ~= current or r.preview ~= preview then
		rt.mgr.ratio = { parent, current, preview }
		ui.render()
	end
end
