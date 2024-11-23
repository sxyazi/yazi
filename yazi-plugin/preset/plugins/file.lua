local M = {}

function M:peek()
	local cmd = os.getenv("YAZI_FILE_ONE") or "file"
	local output, code = Command(cmd):args({ "-bL", "--", tostring(self.file.url) }):stdout(Command.PIPED):output()

	local text
	if output then
		text = ui.Text.parse("----- File Type Classification -----\n\n" .. output.stdout)
	else
		text = ui.Text(string.format("Starting `%s` failed with error code %s. Do you have file(1) installed?", cmd, code))
	end

	ya.preview_widgets(self, { text:area(self.area):wrap(ui.Text.WRAP) })
end

function M:seek() end

function M:spot(job)
	ya.spot_table(
		job,
		ui.Table(self:spot_base(job))
			:area(ui.Pos { "center", w = 60, h = 20 })
			:row(1)
			:col(1)
			:col_style(ui.Style():fg("blue"))
			:cell_style(ui.Style():fg("yellow"):reverse())
			:widths { ui.Constraint.Length(14), ui.Constraint.Fill(1) }
	)
end

function M:spot_base(job)
	local url, cha = job.file.url, job.file.cha
	local spotter = PLUGIN.spotter(url, job.mime)
	local previewer = PLUGIN.previewer(url, job.mime)
	local fetchers = PLUGIN.fetchers(url, job.mime)
	local preloaders = PLUGIN.preloaders(url, job.mime)

	for i, v in ipairs(fetchers) do
		fetchers[i] = v.cmd
	end
	for i, v in ipairs(preloaders) do
		preloaders[i] = v.cmd
	end

	return {
		ui.Row({ "Base" }):style(ui.Style():fg("green")),
		ui.Row { "  Created:", cha.btime and os.date("%y/%m/%d %H:%M", math.floor(cha.btime)) or "-" },
		ui.Row { "  Modified:", cha.mtime and os.date("%y/%m/%d %H:%M", math.floor(cha.mtime)) or "-" },
		ui.Row { "  Mimetype:", job.mime },
		ui.Row {},

		ui.Row({ "Plugins" }):style(ui.Style():fg("green")),
		ui.Row { "  Spotter:", spotter and spotter.cmd or "-" },
		ui.Row { "  Previewer:", previewer and previewer.cmd or "-" },
		ui.Row { "  Fetchers:", #fetchers ~= 0 and fetchers or "-" },
		ui.Row { "  Preloaders:", #preloaders ~= 0 and preloaders or "-" },
	}
end

return M
