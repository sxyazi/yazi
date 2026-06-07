Backdrop = {
	_id = "backdrop",
}

function Backdrop:new(area) return setmetatable({ _area = area }, { __index = self }) end

function Backdrop:reflow() return {} end

function Backdrop:redraw()
	return {
		ui.Fill(self._area):style(th.app.overall),
	}
end
