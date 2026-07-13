local M = {}

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

function M.paste_image(mime, data)
	local type = mime:match("image/([^;]+)")
	local dir = cx.active.current.cwd
	local url = Url(dir .. "/pasted_image." .. type)
	ya.async(function()
		local file = fs.unique("file", url)
		if file then
			fs.write(file, data)
		end
	end)
end

return M
