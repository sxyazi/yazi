local M = {}

function M:spot(job)
	self.size = 0
	self.done = 0
	self.total = #job.files
	self.last = 0
	self:spot_render(job, false)
	local sizes = {}
	for _, url in ipairs(job.files) do
		local cha = fs.cha(url)
		if cha and not cha.is_dir then
			self.size = self.size + cha.len
			self.done = self.done + 1
			self:spot_render(job, false)
		else
			local it = fs.calc_size(url)
			local sub = 0
			while true do
				local next = it:recv()
				if next then
					sub = sub + next
					self.size = self.size + next
					self:spot_render(job, false)
				else
					break
				end
			end
			self.done = self.done + 1
			if cha and cha.is_dir then
				sizes[url.urn] = sub
			end
		end
	end

	if next(sizes) then
		local first = job.files[1]
		local parent = first.parent
		if parent then
			local op = fs.op("size", { url = parent, sizes = sizes })
			ya.emit("update_files", { op = op })
		end
	end
	self:spot_render(job, true)
end

function M:spot_render(job, comp)
	local now = ya.time()
	if not comp and now < self.last + 0.1 then
		return
	end

	local progress = string.format("%d/%d", self.done, self.total)
	local rows = {
		ui.Row({ "Selected" }):style(ui.Style():fg("green")),
		ui.Row { " Count:", tostring(self.total) },
		ui.Row { " Size:", ya.readable_size(self.size) .. (comp and "" or " (?)") },
		ui.Row { " Progress:", comp and "Done" or progress },
	}

	ya.spot_table(
		job,
		ui.Table(rows)
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
