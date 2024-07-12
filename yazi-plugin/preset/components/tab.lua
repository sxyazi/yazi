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
		Rail:new(self._chunks, self._tab),
		Parent:new(self._chunks[1]:padding(ui.Padding.x(1)), self._tab),
		Current:new(self._chunks[2], self._tab),
		Preview:new(self._chunks[3]:padding(ui.Padding.x(1)), self._tab),
	}
end

function Tab:render()
	local children = self._base or {}
	for _, child in ipairs(self._children) do
		children = ya.list_merge(children, ya.render_with(child))
	end
	return children
end

-- Mouse events
function Tab:click(event, up)
	local c = ya.child_at(ui.Position { x = event.x, y = event.y }, self._children)
	return c and c:click(event, up)
end

function Tab:scroll(event, step)
	local c = ya.child_at(ui.Position { x = event.x, y = event.y }, self._children)
	return c and c:scroll(event, step)
end

function Tab:touch(event, step)
	local c = ya.child_at(ui.Position { x = event.x, y = event.y }, self._children)
	return c and c:touch(event, step)
end
