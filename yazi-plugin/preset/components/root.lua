Root = {
	_id = "root",
	_drag_start = ui.Rect {},
	_drag_which = nil, -- "left" or "right" when dragging a panel separator
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
			ui.Constraint.Length(Tabs.height()),
			ui.Constraint.Fill(1),
			ui.Constraint.Length(1),
		})
		:split(self._area)
end

function Root:build()
	self._children = {
		Header:new(self._chunks[1], cx.active),
		Tabs:new(self._chunks[2]),
		Tab:new(self._chunks[3], cx.active),
		Status:new(self._chunks[4], cx.active),
		Modal:new(self._area),
	}
end

function Root:reflow()
	local components = { self }
	for _, child in ipairs(self._children) do
		components = ya.list_merge(components, child:reflow())
	end
	return components
end

function Root:redraw()
	local elements = self._base or {}
	for _, child in ipairs(self._children) do
		elements = ya.list_merge(elements, ui.redraw(child))
	end
	return elements
end

-- Mouse events
function Root:click(event, up)
	if tostring(cx.layer) ~= "mgr" then
		return
	end

	if up then
		Root._drag_which = nil
	elseif event.is_left then
		-- Check if clicking on a panel separator (Tab is self._children[3])
		local tab = self._children[3]
		if tab and tab._chunks then
			local c = tab._chunks
			local lx = c[2].x - 1 -- left separator center x
			local rx = c[3].x -- right separator center x
			if c[1].w > 0 and event.x >= lx - 1 and event.x <= lx + 1 then
				Root._drag_which = "left"
				return
			elseif c[3].w > 0 and event.x >= rx - 1 and event.x <= rx + 1 then
				Root._drag_which = "right"
				return
			end
		end
	end

	local c = ya.child_at(ui.Rect { x = event.x, y = event.y }, self:reflow())
	return c and c:click(event, up)
end

function Root:scroll(event, step)
	if tostring(cx.layer) ~= "mgr" then
		return
	end
	local c = ya.child_at(ui.Rect { x = event.x, y = event.y }, self:reflow())
	return c and c:scroll(event, step)
end

function Root:touch(event, step)
	if tostring(cx.layer) ~= "mgr" then
		return
	end
	local c = ya.child_at(ui.Rect { x = event.x, y = event.y }, self:reflow())
	return c and c:touch(event, step)
end

function Root:move(event) end

function Root:drag(event)
	if not Root._drag_which then
		return
	end

	local tab = self._children[3]
	if not tab or not tab._chunks then
		return
	end

	local c = tab._chunks
	local ratio = rt.mgr.ratio

	if Root._drag_which == "left" then
		-- Dragging the left separator: adjust parent and current widths
		local new_parent = math.max(1, event.x - c[1].x + 1)
		local new_current = math.max(1, c[1].w + c[2].w - new_parent)
		rt.mgr.ratio = { new_parent, new_current, c[3].w > 0 and c[3].w or 1 }
	elseif Root._drag_which == "right" then
		-- Dragging the right separator: adjust current and preview widths
		local right_edge = c[3].x + c[3].w - 1
		local new_preview = math.max(1, right_edge - event.x)
		local new_current = math.max(1, c[2].w + c[3].w - new_preview)
		rt.mgr.ratio = { c[1].w > 0 and c[1].w or 1, new_current, new_preview }
	end
end
