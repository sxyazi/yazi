local M = {}

function M:peek()
	local start, url = os.clock(), ya.file_cache(self)
	if not url or not fs.cha(url) then
		url = self.file.url
	end

	ya.sleep(math.max(0, PREVIEW.image_delay / 1000 + start - os.clock()))
	ya.image_show(url, self.area)
	ya.preview_widgets(self, {})
end

function M:seek() end

function M:preload()
	local cache = ya.file_cache(self)
	if not cache or fs.cha(cache) then
		return 1
	end

	return ya.image_precache(self.file.url, cache) and 1 or 2
end

function M:spot(args)
	local info = ya.image_info(args.file.url)

	local rows = {}
	local row = function(key, value)
		local h = type(value) == "table" and #value or 1
		rows[#rows + 1] = ui.Row({ key, value }):height(h)
	end

	row("Format:", tostring(info.format))
	row("Width:", string.format("%dpx", info.w))
	row("Height:", string.format("%dpx", info.h))
	row("Color:", tostring(info.color))

	ya.spot_table(
		args,
		ui.Table(rows)
			:area(ui.Pos { "center", w = 60, h = 20 })
			:row(args.skip)
			:col(1)
			:col_style(ui.Style():fg("blue"))
			:cell_style(ui.Style():fg("yellow"):reverse())
			:widths { ui.Constraint.Length(12), ui.Constraint.Fill(1) }
	)
end

return M
