Rail = {
	_id = "rail",
}

function Rail:new(chunks, tab)
	local me = setmetatable({ _chunks = chunks, _tab = tab }, { __index = self })
	me:build()
	return me
end

function Rail:build()
	self._base = {
		ui.Bar(ui.Bar.RIGHT):area(self._chunks[1]):symbol(th.manager.border_symbol):style(th.manager.border_style),
		ui.Bar(ui.Bar.LEFT):area(self._chunks[3]):symbol(th.manager.border_symbol):style(th.manager.border_style),
	}
	self._children = {
		Marker:new(self._chunks[1], self._tab.parent),
		Marker:new(self._chunks[2], self._tab.current),
	}
end

function Rail:reflow() return {} end

function Rail:redraw()
	local elements = self._base or {}
	for _, child in ipairs(self._children) do
		elements = ya.list_merge(elements, ya.redraw_with(child))
	end
	return elements
end

-- Mouse events
function Rail:click(event, up) end

function Rail:scroll(event, step) end

function Rail:touch(event, step) end
