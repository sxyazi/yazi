local M = {}

local selected = ya.sync(function()
	local urls = {}
	for _, u in pairs(cx.active.selected) do
		urls[#urls + 1] = u
	end
	return urls
end)

function M:spot(job)
	self.sum, self.sizes, self.last, self.selected = 0, {}, 0, selected()
	self:spot_multi(job, false)

	for _, u in ipairs(self.selected) do
		local it, size = fs.calc_size(u), 0
		while true do
			local next = it:recv()
			if next then
				size, self.sum = size + next, self.sum + next
				self:spot_multi(job, false)
			else
				self.sizes[u], size = size, 0
				break
			end
		end
	end

	self:spot_multi(job, true)
end

function M:spot_multi(job, comp)
	local now = ya.time()
	if not comp and now < self.last + 0.1 then
		return
	end

	local rows = {
		ui.Row({ "Multi" }):style(ui.Style():fg("green")),
		ui.Row { "  Count:", string.format("%d selected", #self.selected) },
		ui.Row { "  Size:", ya.readable_size(self.sum) .. (comp and "" or " (?)") },
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

	self:update_sizes()
	self.last = now
end

function M:update_sizes()
	local parents = {}
	for url, size in pairs(self.sizes) do
		local p = url.parent
		parents[p] = parents[p] or {}
		parents[p][url.urn] = size
	end
	for parent, sizes in pairs(parents) do
		ya.emit("update_files", { op = fs.op("size", { url = parent, sizes = sizes }) })
	end
end

return M
