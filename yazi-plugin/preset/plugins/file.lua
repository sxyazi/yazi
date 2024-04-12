local M = {}

function M:peek()
	local cmd = os.getenv("YAZI_FILE_ONE") or "file"
	local output, code = Command(cmd):args({ "-bL", tostring(self.file.url) }):stdout(Command.PIPED):output()

	local p
	if output then
		p = ui.Paragraph.parse(self.area, "----- File Type Classification -----\n\n" .. output.stdout)
	else
		p = ui.Paragraph(self.area, {
			ui.Line(string.format("Spawn `%s` command returns %s", cmd, code)),
		})
	end

	ya.preview_widgets(self, { p:wrap(ui.Paragraph.WRAP) })
end

function M:seek() end

return M
