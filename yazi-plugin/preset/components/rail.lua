Rail = {
	_id = "rail",
	_area = ui.Rect.default,
}

function Rail:new(chunks, tab)
	local me = setmetatable({ _chunks = chunks, _tab = tab }, { __index = self })
	me:build()
	return me
end

function Rail:build()
	self._base = {
		ui.Bar(self._chunks[1], ui.Bar.RIGHT):symbol(THEME.manager.border_symbol):style(THEME.manager.border_style),
		ui.Bar(self._chunks[3], ui.Bar.LEFT):symbol(THEME.manager.border_symbol):style(THEME.manager.border_style),
	}
	self._children = {
		Marker:new(self._chunks[1], self._tab.parent),
		Marker:new(self._chunks[2], self._tab.current),
	}
end

function Rail:render()
	local children = self._base or {}
	for _, child in ipairs(self._children) do
		children = ya.list_merge(children, ya.render_with(child))
	end
	return children
end

-- Mouse events
function Rail:click(event, up) end

function Rail:scroll(event, step) end

function Rail:touch(event, step) end
