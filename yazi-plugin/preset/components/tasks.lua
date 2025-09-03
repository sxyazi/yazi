Tasks = {
	_id = "tasks",
}

function Tasks:new(area) return setmetatable({ _area = area }, { __index = self }) end

function Tasks:reflow() return { self } end

function Tasks:redraw()
	local rows = {}
	for _, snap in ipairs(cx.tasks.snaps) do
		rows[#rows + 1] = ui.Row { snap.name }
	end

	local tbl = ui.Table(rows)
		:area(self._area:pad(ui.Pad.x(1)))
		:row(cx.tasks.cursor)
		:row_style(th.tasks.hovered)
		:widths { ui.Constraint.Fill(1) }

	return { tbl }
end
