local M = {}

local function date() return os.date("%Y-%m-%d %H-%M-%S") end

function M.selected_uri_list()
	local paths = ya.iter(cx.tab.selected)
		:map(function(f) return "file://" .. ya.percent_encode(tostring(f.path)) end)
		:into()

	if #paths == 0 and cx.tab.current.hovered then
		paths[1] = "file://" .. ya.percent_encode(tostring(cx.tab.current.hovered.path))
	end
	return paths
end

function M.offer_uri_list()
	local list = M.selected_uri_list()
	if #list == 0 then
		return false
	end

	local icon = string.format("%d selected file(s)", #list)
	rt.tty:queue("AgreeDrag", { type = "either", mimes = { "text/uri-list" } })
	rt.tty:queue("PresentDrag", { idx = 0, data = table.concat(list, "\r\n") })
	rt.tty:queue("PresentDragIcon", { format = 0, opacity = 0, width = 6, height = 4, data = icon })
	rt.tty:queue("StartDrag", {})
	rt.tty:flush()
	return true
end

function M.drop(op, mime, data)
	cx.tasks.behavior:reset()

	if mime == "image/png" then
		return M.drop_png(data)
	elseif mime ~= "text/uri-list" then
		return
	end

	for uri in data:gmatch("[^\r\n]+") do
		if uri:sub(1, 7) == "file://" then
			M.drop_file_uri(op, ya.percent_decode(uri:sub(8)))
		elseif uri:match("^https?://") then
			M.drop_http_url(uri)
		elseif uri:sub(1, 5) == "data:" then
			M.drop_data_uri(uri)
		end
	end
end

function M.drop_file_uri(op, path)
	local from = Url(path)
	if from.name then
		local to = cx.tab.current.cwd:join(from.name)
		ya.async(function() ya.task(op, { from = from, to = to }):spawn() end)
	end
end

function M.drop_http_url(url)
	local name = fs.safename(ya.percent_decode(url:match(".*/([^/?#]+)") or "")) or date()
	local to = cx.tab.current.cwd:join(name)

	ya.async(function()
		local task = ya.task("custom", { pool = "plugin", scope = rt.scope() }):name("Drop " .. name):spawn()
		if not task:acquire() then
			return
		end

		local session <close> = ya.http("GET", url):start()
		local resp, err = session:finish()
		if not resp then
			return task:fail(tostring(err))
		elseif resp.status < 200 or resp.status >= 300 then
			return task:fail("HTTP " .. resp.status .. " while downloading " .. url)
		end

		local file <close>, err = fs.unique_file(to)
		if not file then
			return task:fail(tostring(err))
		end

		local ok, err = resp:write(file)
		if ok then
			task:succeed { file.url }
		else
			task:fail(tostring(err))
		end
	end)
end

function M.drop_data_uri(uri)
	local meta, data = uri:match("^data:([^,]*),(.*)$")
	if not data then
		return
	end

	meta = meta:lower()
	local ext = require("clipboard").mime_ext(meta:match("^[^;]*"))
	local to = cx.tab.current.cwd:join(date() .. "." .. ext)

	ya.async(function()
		local task = ya.task("custom", { pool = "plugin", scope = rt.scope() }):name("Drop data"):spawn()
		if not task:acquire() then
			return
		end

		data = ya.percent_decode(data)
		if meta:sub(-7) == ";base64" then
			local err
			data, err = ya.base64_decode(data)
			if not data then
				return task:fail(tostring(err))
			end
		end

		local file <close>, err = fs.unique_file(to)
		if not file then
			return task:fail(tostring(err))
		end

		local ok, err = file:write_all(data)
		if ok then
			ok, err = file:flush()
		end
		if ok then
			task:succeed { file.url }
		else
			task:fail(tostring(err))
		end
	end)
end

function M.drop_png(data)
	local to = cx.tab.current.cwd:join(date() .. ".png")
	ya.async(function()
		local task = ya.task("custom", { pool = "plugin", scope = rt.scope() }):name("Drop image"):spawn()
		if not task:acquire() then
			return
		end

		local file <close>, err = fs.unique_file(to)
		if not file then
			return task:fail(tostring(err))
		end

		local ok, err = file:write_all(data)
		if ok then
			ok, err = file:flush()
		end
		if ok then
			task:succeed { file.url }
		else
			task:fail(tostring(err))
		end
	end)
end

return M
