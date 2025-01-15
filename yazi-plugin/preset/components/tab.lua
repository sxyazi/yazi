Tab = {
	_id = "tab",
}

function Tab:new(area, tab)
	local me = setmetatable({ _area = area, _tab = tab }, { __index = self })
	me:layout()
	me:build()
	return me
end

function Tab:layout()
	self._chunks = ui.Layout()
		:direction(ui.Layout.HORIZONTAL)
		:constraints({
			ui.Constraint.Ratio(MANAGER.ratio.parent, MANAGER.ratio.all),
			ui.Constraint.Ratio(MANAGER.ratio.current, MANAGER.ratio.all),
			ui.Constraint.Ratio(MANAGER.ratio.preview, MANAGER.ratio.all),
		})
		:split(self._area)
end

function Tab:build()
	self._children = {
		Parent:new(self._chunks[1]:pad(ui.Pad.x(1)), self._tab),
		Current:new(self._chunks[2], self._tab),
		Preview:new(self._chunks[3]:pad(ui.Pad.x(1)), self._tab),
		Rail:new(self._chunks, self._tab),
	}
end

function Tab:reflow()
	local components = { self }
	for _, child in ipairs(self._children) do
		components = ya.list_merge(components, child:reflow())
	end
	return components
end

function Tab:redraw()
	local elements = self._base or {}
	for _, child in ipairs(self._children) do
		elements = ya.list_merge(elements, ya.redraw_with(child))
	end
	return elements
end

-- Mouse events
function Tab:click(event, up) end

function Tab:scroll(event, step) end

function Tab:touch(event, step) end
