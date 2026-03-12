local M = {}

local selected = ya.sync(function()
	local urls = {}
	for _, u in pairs(cx.active.selected) do
		urls[#urls + 1] = u
	end
	return urls
end)

function M:spot(job)
	local i = 0
	for rows in self:spot_base(job, selected()) do
		i = i + 1
		ya.spot_table(
			job,
			ui.Table(rows)
				:area(ui.Pos { "center", w = 60, h = 20 })
				:row(i == 1 and 1 or nil)
				:col(1)
				:col_style(th.spot.tbl_col)
				:cell_style(th.spot.tbl_cell)
				:widths { ui.Constraint.Length(14), ui.Constraint.Fill(1) }
		)
		self:update_sizes()
	end
end

function M:spot_base(_, selected)
	local function yield(s)
		coroutine.yield {
			ui.Row({ "Multi" }):style(ui.Style():fg("green")),
			ui.Row { "  Size:", s },
			ui.Row { "  Count:", string.format("%d selected", #selected) },
		}
	end

	self.sizes = {}
	return ya.co(function()
		yield("0B (?)")

		local sum, last = 0, 0
		for _, url in ipairs(selected) do
			local it, size = fs.calc_size(url), 0
			while it do
				local next, now = it:recv(), ya.time()
				if not next then
					self.sizes[url] = it.cha.is_dir and size or nil
					break
				elseif now >= last + 0.1 then
					last, size, sum = now, size + next, sum + next
					yield(ya.readable_size(sum) .. " (?)")
				else
					size, sum = size + next, sum + next
				end
			end
		end

		yield(ya.readable_size(sum))
	end)
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
