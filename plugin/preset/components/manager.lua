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

	return {
		-- Parent
		table.unpack(Folder:render(chunks[1]:padding(ui.Padding.x(1)), { kind = Folder.Kind.Parent })),
		-- Current
		table.unpack(Folder:render(chunks[2], { kind = Folder.Kind.Current })),
		-- Preview
		ui.Base(chunks[3]:padding(ui.Padding.x(1)), "Preview"),

		-- Borders
		ui.Bar(chunks[1], ui.Position.RIGHT):symbol(THEME.manager.border_symbol):style(THEME.manager.border_style),
		ui.Bar(chunks[3], ui.Position.LEFT):symbol(THEME.manager.border_symbol):style(THEME.manager.border_style),
	}
end
