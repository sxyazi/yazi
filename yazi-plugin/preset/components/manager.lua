Manager = {
	area = ui.Rect.default,
}

function Manager:render(area)
	self.area = area

	local chunks = ui.Layout()
		:direction(ui.Layout.HORIZONTAL)
		:constraints({
			ui.Constraint.Ratio(MANAGER.ratio.parent, MANAGER.ratio.all),
			ui.Constraint.Ratio(MANAGER.ratio.current, MANAGER.ratio.all),
			ui.Constraint.Ratio(MANAGER.ratio.preview, MANAGER.ratio.all),
		})
		:split(area)

	return ya.flat {
		-- Borders
		ui.Bar(chunks[1], ui.Bar.RIGHT):symbol(THEME.manager.border_symbol):style(THEME.manager.border_style),
		ui.Bar(chunks[3], ui.Bar.LEFT):symbol(THEME.manager.border_symbol):style(THEME.manager.border_style),

		-- Parent
		Parent:render(chunks[1]:padding(ui.Padding.x(1))),
		-- Current
		Current:render(chunks[2]),
		-- Preview
		Preview:render(chunks[3]:padding(ui.Padding.x(1))),
	}
end
