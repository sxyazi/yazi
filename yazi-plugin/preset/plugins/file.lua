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
	local cha, pair = job.file.cha, { file = job.file, mime = job.mime }
	local spotter, previewer, fetchers, preloaders = nil, nil, {}, {}

	for _, v in pairs(rt.plugin.spotters:match(pair)) do
		spotter = v
		break
	end

	for _, v in pairs(rt.plugin.previewers:match(pair)) do
		previewer = v
		break
	end

	for _, v in pairs(rt.plugin.fetchers:match(pair)) do
		fetchers[#fetchers + 1] = v.name
	end
	fetchers = #fetchers ~= 0 and fetchers or { "-" }

	for _, v in pairs(rt.plugin.preloaders:match(pair)) do
		preloaders[#preloaders + 1] = v.name
	end
	preloaders = #preloaders ~= 0 and preloaders or { "-" }

	return {
		ui.Row({ "Base" }):style(ui.Style():fg("green")),
		ui.Row { "  Created:", cha.btime and os.date("%Y-%m-%d %H:%M:%S", math.floor(cha.btime)) or "-" },
		ui.Row { "  Modified:", cha.mtime and os.date("%Y-%m-%d %H:%M:%S", math.floor(cha.mtime)) or "-" },
		ui.Row { "  Mimetype:", job.mime },
		ui.Row {},

		ui.Row({ "Plugins" }):style(ui.Style():fg("green")),
		ui.Row { "  Spotter:", spotter and spotter.name or "-" },
		ui.Row { "  Previewer:", previewer and previewer.name or "-" },
		ui.Row({ "  Fetchers:", fetchers }):height(#fetchers),
		ui.Row({ "  Preloaders:", preloaders }):height(#preloaders),
	}
end

return M
