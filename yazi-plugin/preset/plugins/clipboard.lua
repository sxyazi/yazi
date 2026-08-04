local M = {}

function M.read_unsolicited(mime, event)
	rt.tty:queue("ReadClipboard", {
		mimes = { mime },
		pw = event.pw,
		name = "Paste event",
		primary = event.primary,
	})
	rt.tty:flush()
end

function M.copy_uri_list(list)
	cx.tasks.behavior:reset()
	for line in list:gmatch("[^\r\n]+") do
		if line:sub(1, 7) ~= "file://" then
			goto continue
		end

		local from = Url(ya.percent_decode(line:sub(8)))
		if from.name then
			local to = cx.active.current.cwd:join(from.name)
			ya.async(function() ya.task("copy", { from = from, to = to }):spawn() end)
		end

		::continue::
	end
end

return M
