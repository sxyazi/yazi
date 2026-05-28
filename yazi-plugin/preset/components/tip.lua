Tip = {
	_id = "tip",
}

function Tip:new(area, text)
	local me = setmetatable({ _area = area, _text = text }, { __index = self })
	me:layout()
	return me
end

function Tip:layout()
	self._chunks = ui.Layout()
		:direction(ui.Layout.VERTICAL)
		:constraints({
			ui.Constraint.Fill(1),
			ui.Constraint.Length(1),
			ui.Constraint.Fill(1),
		})
		:split(self._area)
end

function Tip:reflow() return {} end

function Tip:redraw()
	return {
		ui.Clear(self._chunks[2]),
		ui.Text(self._text):area(self._chunks[2]):align(ui.Align.CENTER):fg("black"):bg("yellow"):bold(),
	}
end
