local M = {}

function M:peek()
	local output, code = Command("file")
		:args({
			"-bL",
			tostring(self.file.url),
		})
		:stdout(Command.PIPED)
		:output()

	local p
	if output then
		p = ui.Paragraph.parse(self.area, "----- File Type Classification -----\n\n" .. output.stdout)
	else
		p = ui.Paragraph(self.area, {
			ui.Line {
				ui.Span("Failed to spawn `file` command, error code: " .. tostring(code)),
			},
		})
	end

	ya.preview_widgets(self, { p:wrap(ui.Paragraph.WRAP) })
end

function M:seek() end

return M
