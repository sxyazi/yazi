Rail = {
	_id = "rail",
	_area = ui.Rect.default,
}

function Rail:new(chunks, tab)
	return setmetatable({
		_chunks = chunks,
		_tab = tab,
	}, { __index = self }):build()
end

function Rail:build()
	self._bars = {
		ui.Bar(self._chunks[1], ui.Bar.RIGHT):symbol(THEME.manager.border_symbol):style(THEME.manager.border_style),
		ui.Bar(self._chunks[3], ui.Bar.LEFT):symbol(THEME.manager.border_symbol):style(THEME.manager.border_style),
	}
	self._children = {
		Marker:new(self._chunks[1], self._tab.parent),
		Marker:new(self._chunks[2], self._tab.current),
	}
	return self
end

function Rail:render()
	local result = self._bars
	for _, child in ipairs(self._children) do
		result = ya.list_merge(result, ya.render_with(child))
	end
	return result
end

-- Mouse events
function Rail:click(event, up) end

function Rail:scroll(event, step) end

function Rail:touch(event, step) end
