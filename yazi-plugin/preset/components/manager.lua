Manager = {
	area = ui.Rect.default,
}

function Manager:render(area)
	self.area = area

	local chunks = ui.Layout()
		:direction(ui.Direction.HORIZONTAL)
		:constraints({
			ui.Constraint.Ratio(MANAGER.layout.parent, MANAGER.layout.all),
			ui.Constraint.Ratio(MANAGER.layout.current, MANAGER.layout.all),
			ui.Constraint.Ratio(MANAGER.layout.preview, MANAGER.layout.all),
		})
		:split(area)

	return ya.flat {
		-- Borders
		ui.Bar(chunks[1], ui.Position.RIGHT):symbol(THEME.manager.border_symbol):style(THEME.manager.border_style),
		ui.Bar(chunks[3], ui.Position.LEFT):symbol(THEME.manager.border_symbol):style(THEME.manager.border_style),

		-- Parent
		Parent:render(chunks[1]:padding(ui.Padding.x(1))),
		-- Current
		Current:render(chunks[2]),
		-- Preview
		Preview:render(chunks[3]:padding(ui.Padding.x(1))),
	}
end
