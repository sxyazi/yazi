Root = {
	_id = "root",
	_dragging = nil,
	_dropping = nil,
	_reading = false,
	_pastes = {},
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

function Root:read_clipboard(paste)
	Root._reading = true
	rt.tty:queue(
		"ReadClipboard",
		{ mimes = { paste.mime }, pw = paste.pw, name = "Paste event", primary = paste.primary }
	)
	rt.tty:flush()
end

function Root:read_next_clipboard()
	local paste = table.remove(Root._pastes, 1)
	if paste then
		Root:read_clipboard(paste)
	end
end

function Root:clipboard(event)
	if not event then
		return
	elseif event.type == "read" then
		local mimes = event.data["."]
		if mimes then
			local mime
			for m in mimes:gmatch("%S+") do
				if m == "text/plain" then
					mime = m
					break
				elseif m == "text/uri-list" then
					mime = m
				end
			end
			if mime then
				Root._pastes[#Root._pastes + 1] = { mime = mime, pw = event.pw, primary = event.primary }
				if not Root._reading then
					Root:read_next_clipboard()
				end
			end
			return
		elseif not Root._reading then
			return
		end

		Root._reading = false
		local text = event.data["text/plain"]
		if tostring(cx.layer) == "input" and text ~= nil then
			ya.emit("input:feed", { text })
		elseif event.data["text/uri-list"] ~= nil then
			require("clipboard").copy_uri_list(event.data["text/uri-list"])
		end
	elseif event.type == "error" and not event.write then
		Root._reading = false
	else
		return
	end

	Root:read_next_clipboard()
end
