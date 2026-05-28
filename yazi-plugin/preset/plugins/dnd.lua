local M = {}

function M.selected_uri_list()
	local paths = {}
	for _, u in pairs(cx.active.selected) do
		paths[#paths + 1] = "file://" .. ya.percent_encode(tostring(u.path))
	end
	if #paths == 0 and cx.active.current.hovered then
		paths[1] = "file://" .. ya.percent_encode(tostring(cx.active.current.hovered.path))
	end
	return paths
end

function M.cut_uri_list(list)
	cx.tasks.behavior:reset()
	for line in list:gmatch("[^\r\n]+") do
		if line:sub(1, 7) ~= "file://" then
			goto continue
		end

		local from = Url(ya.percent_decode(line:sub(8)))
		if from.name then
			local to = cx.active.current.cwd:join(from.name)
			ya.async(function() ya.task("cut", { from = from, to = to }):spawn() end)
		end

		::continue::
	end
end

function M.offer_uri_list()
	local list = M.selected_uri_list()
	if #list == 0 then
		return false
	end

	local icon = string.format("%d selected file(s)", #list)
	rt.tty:queue("ConfirmDrag", { type = "either", mimes = { "text/uri-list" } })
	rt.tty:queue("PresentDrag", { idx = 0, data = table.concat(list, "\r\n") })
	rt.tty:queue("PresentDragIcon", { format = 0, opacity = 0, width = 6, height = 4, data = icon })
	rt.tty:queue("StartDrag", {})
	rt.tty:flush()
	return true
end

return M
