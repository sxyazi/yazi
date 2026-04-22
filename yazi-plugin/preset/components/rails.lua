Rails = {
	_id = "rails",
}

function Rails:new(chunks, tab)
	local me = setmetatable({ _chunks = chunks, _tab = tab }, { __index = self })
	me:build()
	return me
end

function Rails:build()
	local c, children = self._chunks, {}
	if c[1].w > 0 then
		children[#children + 1] = Rail:new("rail-left", c[2] { w = math.min(1, c[2].w) }, c)
	end
	if c[3].w > 0 then
		children[#children + 1] =
			Rail:new("rail-right", c[2] { x = math.max(0, c[2].right - 1), w = math.min(1, c[2].w) }, c)
	end
	self._children = children
end

function Rails:reflow()
	local components = {}
	for _, child in ipairs(self._children) do
		components = ya.list_merge(components, child:reflow())
	end
	return components
end

function Rails:redraw()
	local elements = {}
	for _, child in ipairs(self._children) do
		elements = ya.list_merge(elements, ui.redraw(child))
	end
	return elements
end

-- Mouse events
function Rails:click(event, up) end

function Rails:scroll(event, step) end

function Rails:touch(event, step) end
