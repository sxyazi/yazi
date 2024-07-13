Root = {
	_id = "root",
	_drag_start = ui.Rect.default,
}

function Root:new(area)
	local me = setmetatable({ _area = area }, { __index = self })
	me:layout()
	me:build()
	return me
end

function Root:layout()
	self._chunks = ui.Layout()
		:direction(ui.Layout.VERTICAL)
		:constraints({
			ui.Constraint.Length(1),
			ui.Constraint.Fill(1),
			ui.Constraint.Length(1),
		})
		:split(self._area)
end

function Root:build()
	self._children = {
		Header:new(self._chunks[1], cx.active),
		Tab:new(self._chunks[2], cx.active),
		Status:new(self._chunks[3], cx.active),
	}
end

function Root:render()
	local children = self._base or {}
	for _, child in ipairs(self._children) do
		children = ya.list_merge(children, ya.render_with(child))
	end
	__yazi_check_and_warn_deprecated_api() -- TODO: remove this after 0.3.0 release
	return children
end

-- Mouse events
function Root:click(event, up)
	local c = ya.child_at(ui.Position { x = event.x, y = event.y }, self._children)
	return c and c:click(event, up)
end

function Root:scroll(event, step)
	local c = ya.child_at(ui.Position { x = event.x, y = event.y }, self._children)
	return c and c:scroll(event, step)
end

function Root:touch(event, step)
	local c = ya.child_at(ui.Position { x = event.x, y = event.y }, self._children)
	return c and c:touch(event, step)
end

function Root:move(event) end

function Root:drag(event) end
