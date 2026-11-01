Dnd = {
	_op = "reject",
	_allowed = "copy",
	_mime = nil,
	_mimes = nil,
	_idx = nil,
	_dragging = false,
	_dropping = false,
}

local function centered(area, height)
	return ui.Layout()
		:direction(ui.Layout.VERTICAL)
		:constraints({ ui.Constraint.Fill(1), ui.Constraint.Length(height), ui.Constraint.Fill(1) })
		:split(area)[2]
end

function Dnd:new(area)
	local me = setmetatable({ _area = area }, { __index = self })
	me:layout()
	me:build()
	return me
end

function Dnd:layout()
	if Dnd._allowed ~= "either" then
		self._zones = { centered(self._area, 3) }
		return
	end

	local chunks = ui.Layout()
		:direction(ui.Layout.VERTICAL)
		:constraints({
			ui.Constraint.Fill(1),
			ui.Constraint.Length(3),
			ui.Constraint.Length(2),
			ui.Constraint.Length(3),
			ui.Constraint.Fill(1),
		})
		:split(self._area)

	self._zones = { chunks[2], chunks[4] }
end

function Dnd:build()
	if #self._zones == 1 then
		self._items = { { op = Dnd._allowed, area = self._zones[1] } }
	else
		self._items = { { op = "copy", area = self._zones[1] }, { op = "move", area = self._zones[2] } }
	end
end

function Dnd:redraw()
	if not Dnd._dropping then
		return {}
	end

	local elements = {}
	for _, item in ipairs(self._items) do
		local active = item.op == Dnd._op
		elements[#elements + 1] = ui.Clear(item.area)
		elements[#elements + 1] = ui.Text(""):area(item.area):bg(active and "yellow" or "darkgray")
		elements[#elements + 1] = ui.Text("Drop to " .. item.op .. " here…")
			:area(centered(item.area, 1):pad(ui.Pad.x(1)))
			:align(ui.Align.CENTER)
			:fg(active and "black" or "white")
			:bold()
	end
	return elements
end

function Dnd.drag(event)
	if event.type == "offer" then
		Dnd._dragging = require("dnd").offer_uri_list()
	elseif event.type == "end" or event.type == "error" then
		Dnd._dragging = false
	end
end

function Dnd.drop(event, area)
	if Dnd._dragging then
		return false
	end

	local op, dropping = Dnd._op, Dnd._dropping
	if event.type == "enter" then
		Dnd.consume_mimes()
		Dnd._allowed = event.op
		Dnd._op = Dnd._idx and Dnd:new(area):op_at(event.x, event.y) or "reject"
		rt.tty:queue("AgreeDrop", { type = Dnd._op, mimes = Dnd._mime and { Dnd._mime } or {} })
	elseif event.type == "ready" then
		Dnd.consume_mimes()
		local kind = Dnd._idx and "StartDrop" or "FinishDrop"
		local data = Dnd._idx and { idx = Dnd._idx } or { type = "reject" }
		rt.tty:queue(kind, data)
	elseif event.type == "arrive" then
		rt.tty:queue("FinishDrop", { type = Dnd._op })
		require("dnd").drop(Dnd._op, Dnd._mime, event.data)
	end
	rt.tty:flush()

	Dnd._dropping = event.type == "enter"
	return Dnd._op ~= op or Dnd._dropping ~= dropping
end

function Dnd:op_at(x, y)
	local point = ui.Rect { x = x, y = y }
	for _, item in ipairs(self._items) do
		if item.area:contains(point) then
			return item.op
		end
	end
	return "reject"
end

function Dnd.consume_mimes()
	if not Dnd._mimes then
		return
	end
	for _, wanted in ipairs { "text/uri-list", "image/png" } do
		for i, mime in ipairs(Dnd._mimes) do
			if mime == wanted then
				Dnd._idx, Dnd._mime, Dnd._mimes = i, mime, nil
				return
			end
		end
	end
	Dnd._idx, Dnd._mime, Dnd._mimes = nil, nil, nil
end
