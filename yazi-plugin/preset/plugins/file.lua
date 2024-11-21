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

-- TODO: remove this
local hovered_mime = ya.sync(function()
	local h = cx.active.current.hovered
	if not h then
		return nil
	elseif h.cha.is_dir then
		return "inode/directory"
	else
		return h:mime()
	end
end)

function M:spot(job)
	local mime = hovered_mime()
	if not mime then
		return
	end

	local file = job.file
	local spotter = PLUGIN.spotter(file.url, mime)
	local previewer = PLUGIN.previewer(file.url, mime)
	local fetchers = PLUGIN.fetchers(file.url, mime)
	local preloaders = PLUGIN.preloaders(file.url, mime)

	for i, v in ipairs(fetchers) do
		fetchers[i] = v.cmd
	end
	for i, v in ipairs(preloaders) do
		preloaders[i] = v.cmd
	end

	local rows = {}
	local row = function(key, value)
		local h = type(value) == "table" and #value or 1
		rows[#rows + 1] = ui.Row({ key, value }):height(h)
	end

	rows[#rows + 1] = ui.Row({ "Metadata", "" }):style(ui.Style():fg("red"))
	row("  Created:", file.cha.btime and os.date("%y/%m/%d %H:%M", math.floor(file.cha.btime)) or "-")
	row("  Modified:", file.cha.mtime and os.date("%y/%m/%d %H:%M", math.floor(file.cha.mtime)) or "-")
	row("  Mimetype:", mime)
	rows[#rows + 1] = ui.Row({ { "", "Plugins" }, "" }):height(2):style(ui.Style():fg("red"))
	row("  Spotter:", spotter and spotter.cmd or "-")
	row("  Previewer:", previewer and previewer.cmd or "-")
	row("  Fetchers:", #fetchers ~= 0 and fetchers or "-")
	row("  Preloaders:", #preloaders ~= 0 and preloaders or "-")

	ya.spot_table(
		job,
		ui.Table(rows)
			:area(ui.Pos { "center", w = 60, h = 20 })
			:row(1)
			:col(1)
			:col_style(ui.Style():fg("blue"))
			:cell_style(ui.Style():fg("yellow"):reverse())
			:widths { ui.Constraint.Length(14), ui.Constraint.Fill(1) }
	)
end

return M
