local M = {}

function M:peek()
	local cmd = os.getenv("YAZI_FILE_ONE") or "file"
	local output, code = Command(cmd):args({ "-bL", "--", tostring(self.file.url) }):stdout(Command.PIPED):output()

	local text
	if output then
		text = ui.Text.parse("----- File Type Classification -----\n\n" .. output.stdout)
	else
		text = ui.Text(string.format("Spawn `%s` command returns %s", cmd, code))
	end

	ya.preview_widgets(self, { text:area(self.area):wrap(ui.Text.WRAP) })
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
