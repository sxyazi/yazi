local M = {}

function M:peek(job)
	local program = os.getenv("YAZI_FILE_ONE") or "file"
	local path = tostring(job.file.path)
	local output, err = Command(program):arg({ "-bL", "--", path }):output()

	local text
	if output then
		text = ui.Text.parse("----- File Type Classification -----\n\n" .. output.stdout)
	else
		text = ui.Text(string.format("Failed to start `%s`, error: %s", program, err))
	end

	ya.preview_widget(job, text:area(job.area):wrap(ui.Wrap.YES))
end

function M:seek() end

function M:spot(job)
	ya.spot_table(
		job,
		ui.Table(self:spot_base(job))
			:area(ui.Pos { "center", w = 60, h = 20 })
			:row(1)
			:col(1)
			:col_style(th.spot.tbl_col)
			:cell_style(th.spot.tbl_cell)
			:widths { ui.Constraint.Length(14), ui.Constraint.Fill(1) }
	)
end

function M:spot_base(job)
	local stat, pair = job.file.stat, { file = job.file, mime = job.mime }

	local spotter = rt.plugin.spotters:match(pair)()
	local previewer = rt.plugin.previewers:match(pair)()

	local fetchers = ya.iter(rt.plugin.fetchers:match(pair)):map(function(v) return v.run.name end):into()
	fetchers = #fetchers ~= 0 and fetchers or { "-" }

	local preloaders = ya.iter(rt.plugin.preloaders:match(pair)):map(function(v) return v.run.name end):into()
	preloaders = #preloaders ~= 0 and preloaders or { "-" }

	return {
		ui.Row({ "Base" }):style(ui.Style():fg("green")),
		ui.Row { "  Created:", stat.btime and tostring(stat.btime) or "-" },
		ui.Row { "  Modified:", stat.mtime and tostring(stat.mtime) or "-" },
		ui.Row { "  Mimetype:", job.mime },
		ui.Row {},

		ui.Row({ "Plugins" }):style(ui.Style():fg("green")),
		ui.Row { "  Spotter:", spotter and spotter.run.name or "-" },
		ui.Row { "  Previewer:", previewer and previewer.run.name or "-" },
		ui.Row({ "  Fetchers:", fetchers }):height(#fetchers),
		ui.Row({ "  Preloaders:", preloaders }):height(#preloaders),
	}
end

return M
