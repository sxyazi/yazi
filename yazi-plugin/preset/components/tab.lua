Manager = {} -- TODO: remove this after 0.3.0 release
Tab = {
	_id = "tab",
}

function Tab:new(area, tab)
	return setmetatable({
		_area = area,
		_tab = tab,
	}, { __index = self }):build()
end

function Tab:build()
	local chunks = self:layout()

	self._children = {
		Rail:new(chunks, self._tab),
		Parent:new(chunks[1]:padding(ui.Padding.x(1)), self._tab),
		Current:new(chunks[2], self._tab),
		Preview:new(chunks[3]:padding(ui.Padding.x(1)), self._tab),
	}
	return self
end

function Tab:render()
	local children = {}
	for _, child in ipairs(self._children) do
		children = ya.list_merge(children, ya.render_with(child))
	end
	return children
end

function Tab:layout()
	return ui.Layout()
		:direction(ui.Layout.HORIZONTAL)
		:constraints({
			ui.Constraint.Ratio(MANAGER.ratio.parent, MANAGER.ratio.all),
			ui.Constraint.Ratio(MANAGER.ratio.current, MANAGER.ratio.all),
			ui.Constraint.Ratio(MANAGER.ratio.preview, MANAGER.ratio.all),
		})
		:split(self._area)
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
