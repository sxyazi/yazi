Manager = {}

function Manager:render(area)
	local chunks = ui.Layout()
		:direction(ui.Direction.HORIZONTAL)
		:constraints({
			ui.Constraint.Ratio(MANAGER.layout.parent, MANAGER.layout.all),
			ui.Constraint.Ratio(MANAGER.layout.current, MANAGER.layout.all),
			ui.Constraint.Ratio(MANAGER.layout.preview, MANAGER.layout.all),
		})
		:split(area)

	return utils.flat {
		-- Borders
		ui.Bar(chunks[1], ui.Position.RIGHT):symbol(THEME.manager.border_symbol):style(THEME.manager.border_style),
		ui.Bar(chunks[3], ui.Position.LEFT):symbol(THEME.manager.border_symbol):style(THEME.manager.border_style),

		-- Parent
		Folder:render(chunks[1]:padding(ui.Padding.x(1)), { kind = Folder.PARENT }),
		-- Current
		Folder:render(chunks[2], { kind = Folder.CURRENT }),
		-- Preview
		ui.Base(chunks[3]:padding(ui.Padding.x(1)), ui.Base.PREVIEW),
	}
end
