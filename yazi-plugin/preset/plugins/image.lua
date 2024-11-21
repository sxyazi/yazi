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

function M:spot(job)
	local rows = self:spot_base(job)
	rows[#rows + 1] = ui.Row {}

	ya.spot_table(
		job,
		ui.Table(ya.list_merge(rows, require("file"):spot_base(job)))
			:area(ui.Pos { "center", w = 60, h = 20 })
			:row(job.skip)
			:row(1)
			:col(1)
			:col_style(ui.Style():fg("blue"))
			:cell_style(ui.Style():fg("yellow"):reverse())
			:widths { ui.Constraint.Length(14), ui.Constraint.Fill(1) }
	)
end

function M:spot_base(job)
	local info = ya.image_info(job.file.url)
	return {
		ui.Row({ "Image" }):style(ui.Style():fg("green")),
		ui.Row { "  Format:", tostring(info.format) },
		ui.Row { "  Size:", string.format("%dx%d", info.w, info.h) },
		ui.Row { "  Color:", tostring(info.color) },
	}
end

return M
