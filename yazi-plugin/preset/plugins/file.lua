local M = {}

function M:peek(job)
	local cmd = os.getenv("YAZI_FILE_ONE") or "file"
	local output, err = Command(cmd):args({ "-bL", "--", tostring(job.file.url) }):stdout(Command.PIPED):output()

	local text
	if output then
		text = ui.Text.parse("----- File Type Classification -----\n\n" .. output.stdout)
	else
		text = ui.Text(string.format("Failed to start `%s`, error: %s", cmd, err))
	end

	ya.preview_widgets(job, { text:area(job.area):wrap(ui.Text.WRAP) })
end

function M:seek() end

function M:spot(job)
	ya.spot_table(
		job,
		ui.Table(self:spot_base(job))
			:area(ui.Pos { "center", w = 60, h = 20 })
			:row(1)
			:col(1)
			:col_style(THEME.spot.values)
			:cell_style(THEME.spot.selected)
			:widths { ui.Constraint.Length(14), ui.Constraint.Fill(1) }
	)
end

function M:spot_base(job)
	local url, cha = job.file.url, job.file.cha
	local spotter = PLUGIN.spotter(url, job.mime)
	local previewer = PLUGIN.previewer(url, job.mime)
	local fetchers = PLUGIN.fetchers(job.file, job.mime)
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
