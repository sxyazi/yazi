local M = {}

local selected = ya.sync(function()
	return ya.iter(pairs(cx.tab.selected)):map(function(_, f) return f.url end):into()
end)

function M:spot(job)
	for i, rows in ya.iter(self:spot_base(job, selected())):enumerate() do
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

		local sum, tick = 0, ya.ticker(0.1)
		for _, url in ipairs(selected) do
			local it, size = fs.calc_size(url), 0
			if not it then
				return yield("Error")
			end

			for next in it do
				if not next then
					return yield("Error")
				end

				size, sum = size + next, sum + next
				if tick() then
					yield(ya.readable_size(sum) .. " (?)")
				end
			end
			self.sizes[url] = it.lstat.is_dir and size or nil
		end

		yield(ya.readable_size(sum))
	end)
end

function M:update_sizes()
	local trails = {}
	for url, size in pairs(self.sizes) do
		local t = url.trail
		trails[t] = trails[t] or {}
		trails[t][url.key] = size
	end
	for trail, sizes in pairs(trails) do
		ya.emit("update_files", { op = fs.op("size", { url = trail, sizes = sizes }) })
	end
end

return M
