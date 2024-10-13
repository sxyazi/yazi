local M = {}

function M:peek()
	local cmd = os.getenv("YAZI_FILE_ONE") or "file"
	local output, code = Command(cmd):args({ "-bL", tostring(self.file.url) }):stdout(Command.PIPED):output()

	local p
	if output then
		p = ui.Text.parse("----- File Type Classification -----\n\n" .. output.stdout):area(self.area)
	else
		p = ui.Text(string.format("Spawn `%s` command returns %s", cmd, code)):area(self.area)
	end

	ya.preview_widgets(self, { p:wrap(ui.Text.WRAP) })
end

function M:seek() end

function M:spot(skip)
	local rect = ui.Rect { x = 10, y = 10, w = 20, h = 20 }

	ya.spot_widgets(self, {
		ui.Clear(rect),
		ui.Table(rect),
	})
end

return M
