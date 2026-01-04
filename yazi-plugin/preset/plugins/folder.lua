--- @sync peek

local M = {}

function M:peek(job)
	local folder = cx.active.preview.folder
	if not folder then
		return ya.preview_widget(job, ui.Line("Loading..."):area(job.area):align(ui.Align.CENTER))
	elseif folder.cwd ~= job.file.url then
		return
	end

	local bound = math.max(0, #folder.files - job.area.h)
	if job.skip > bound then
		return ya.emit("peek", { bound, only_if = job.file.url, upper_bound = true })
	end

	if #folder.files == 0 then
		local done, err = folder.stage()
		local s = not done and "Loading..." or not err and "No items" or string.format("Error: %s", err)
		return ya.preview_widget(job, ui.Text(s):area(job.area):align(ui.Align.CENTER):wrap(ui.Wrap.YES))
	end

	local left, right = {}, {}
	for _, f in ipairs(folder.window) do
		local entity = Entity:new(f)
		left[#left + 1], right[#right + 1] = entity:redraw(), Linemode:new(f):redraw()

		local max = math.max(0, job.area.w - right[#right]:width())
		left[#left]:truncate { max = max, ellipsis = entity:ellipsis(max) }
	end

	ya.preview_widget(job, {
		ui.List(left):area(job.area),
		ui.Text(right):area(job.area):align(ui.Align.RIGHT),
		table.unpack(Marker:new(job.area, folder):redraw()),
	})
end

function M:seek(job)
	local folder = cx.active.preview.folder
	if folder and folder.cwd == job.file.url then
		local step = math.floor(job.units * job.area.h / 10)
		local bound = math.max(0, #folder.files - job.area.h)
		ya.emit("peek", {
			ya.clamp(0, cx.active.preview.skip + step, bound),
			only_if = job.file.url,
		})
	end
end

function M:spot(job)
	self.size, self.last = 0, 0
	self:spot_multi(job, false)

	local url = job.file.url
	local it = fs.calc_size(url)
	while true do
		local next = it:recv()
		if next then
			self.size = self.size + next
			self:spot_multi(job, false)
		else
			break
		end
	end

	local op = fs.op("size", { url = url.parent, sizes = { [url.urn] = self.size } })
	ya.emit("update_files", { op = op })

	self:spot_multi(job, true)
end

function M:spot_multi(job, comp)
	local now = ya.time()
	if not comp and now < self.last + 0.1 then
		return
	end

	local rows = {
		ui.Row({ "Folder" }):style(ui.Style():fg("green")),
		ui.Row { "  Size:", ya.readable_size(self.size) .. (comp and "" or " (?)") },
		ui.Row {},
	}

	ya.spot_table(
		job,
		ui.Table(ya.list_merge(rows, require("file"):spot_base(job)))
			:area(ui.Pos { "center", w = 60, h = 20 })
			:row(self.last == 0 and 1 or nil)
			:col(1)
			:col_style(th.spot.tbl_col)
			:cell_style(th.spot.tbl_cell)
			:widths { ui.Constraint.Length(14), ui.Constraint.Fill(1) }
	)
	self.last = now
end

return M
