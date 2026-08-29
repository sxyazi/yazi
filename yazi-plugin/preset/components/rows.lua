Rows = {}

-- Picks the row style for a file; override in init.lua for per-dir or time-based rules.
function Rows.pick(file)
	local pane = file.in_current and th.mgr.rows_current or file.in_preview and th.mgr.rows_preview or th.mgr.rows_parent
	local rows = #pane > 0 and pane or th.mgr.rows
	if #rows > 0 then
		return rows[(file.idx % #rows) + 1]
	end
end
