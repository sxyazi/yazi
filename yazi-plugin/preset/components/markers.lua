Markers = {
	_id = "markers",
}

function Markers:new(chunks, tab)
	local me = setmetatable({ _chunks = chunks, _tab = tab }, { __index = self })
	me:build()
	return me
end

function Markers:build()
	self._children = {
		Marker:new(self._chunks[1], self._tab.parent),
		Marker:new(self._chunks[2], self._tab.current),
	}
end

function Markers:reflow() return {} end

function Markers:redraw()
	local elements = {}
	for _, child in ipairs(self._children) do
		elements = ya.list_merge(elements, ui.redraw(child))
	end
	return elements
end

-- Mouse events
function Markers:click(event, up) end

function Markers:scroll(event, step) end

function Markers:touch(event, step) end
