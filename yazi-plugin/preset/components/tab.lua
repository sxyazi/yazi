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
	local ratio = rt.mgr.ratio
	self._chunks = ui.Layout()
		:direction(ui.Layout.HORIZONTAL)
		:constraints({
			ui.Constraint.Ratio(ratio.parent, ratio.all),
			ui.Constraint.Ratio(ratio.current, ratio.all),
			ui.Constraint.Ratio(ratio.preview, ratio.all),
		})
		:split(self._area)
end

function Tab:build()
	local c = self._chunks
	self._children = {
		Parent:new(c[1]:pad(ui.Pad.x(1)), self._tab),
		Current:new(c[2]:pad(ui.Pad(0, c[3].w > 0 and 0 or 1, 0, c[1].w > 0 and 0 or 1)), self._tab),
		Preview:new(c[3]:pad(ui.Pad.x(1)), self._tab),
		Rail:new(c, self._tab),
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
		elements = ya.list_merge(elements, ui.redraw(child))
	end
	return elements
end

-- Mouse events
function Tab:click(event, up) end

function Tab:scroll(event, step) end

function Tab:touch(event, step) end
