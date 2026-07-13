Root = {
	_id = "root",
	_dragging = nil,
	_dropping = nil,
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
		Backdrop:new(self._area),
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
	local c = ya.child_at(ui.Rect { x = event.x, y = event.y }, self:reflow())
	Root._dragging = not up and c or nil

	if tostring(cx.layer) == "mgr" then
		return c and c.click and c:click(event, up)
	end
end

function Root:scroll(event, step)
	if tostring(cx.layer) ~= "mgr" then
		return
	end
	local c = ya.child_at(ui.Rect { x = event.x, y = event.y }, self:reflow())
	return c and c.scroll and c:scroll(event, step)
end

function Root:touch(event, step)
	if tostring(cx.layer) ~= "mgr" then
		return
	end
	local c = ya.child_at(ui.Rect { x = event.x, y = event.y }, self:reflow())
	return c and c.touch and c:touch(event, step)
end

function Root:move(event) end

function Root:drag(event)
	if tostring(cx.layer) ~= "mgr" then
		return
	end

	local c = Root._dragging
	return c and c.drag and c:drag(event)
end

function Root:drop(event)
	local d = Root._dropping
	local c = event.x and ya.child_at(ui.Rect { x = event.x, y = event.y }, self:reflow()) or d
	if d and d.drop and d._id ~= c._id then
		d:drop { type = "leave" }
	end

	Root._dropping = c
	if tostring(cx.layer) == "mgr" then
		return c and c.drop and c:drop(event)
	end
end

-- Clipboard events
function Root:paste_offer(event)
	if event and event.pw then
		-- No harm in asking for unavailable types
		local mimetypes = "text/plain text/uri-list"
		ya.dbg("Requesting ReadClipboard")
		rt.tty:queue("ReadClipboard", { mimes = mimetypes, pw = event.pw, name = "Paste Event", primary = event.primary })
		rt.tty:flush()
	end
end

function Root:paste_data(event)
	if event.data["text/uri-list"] ~= nil then
		local list = event.data["text/uri-list"]
		ya.dbg("Pasting URI list:", list)
		require("clipboard").copy_uri_list(list)
	end
	-- TODO !!5522!! Suport non text formats
	-- if event.data["image/png"] ~= nil then
	-- 	local type = "image/png"
	-- 	local data = event.data["image/png"]
	-- 	ya.dbg("Pasting image/png:")
	-- 	require("clipboard").paste_image(type, data)
	-- end
end

function Root:write_result(event) end
