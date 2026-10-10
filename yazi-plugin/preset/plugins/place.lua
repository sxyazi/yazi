local M = {}

local function target(url)
	local path = tostring(url.path)
	if path:match("^/[A-Za-z]:$") then
		return Url(path:sub(2) .. "\\") -- /C: -> C:\
	elseif path:match("^/[A-Za-z]:/") then
		return Url(path:sub(2)) -- /C:/Users -> C:/Users
	end
end

local function stat(url, follow)
	local path = tostring(url.path)
	if path == "/" or path:match("^/[A-Za-z]:$") then
		return Stat { mode = tonumber("40555", 8) }
	end

	local source = target(url)
	if not source then
		return nil, Error.fs { kind = "NotFound", message = "Invalid place location" }
	end

	return fs.stat(source, follow)
end

local function file(url)
	local path = tostring(url.path)
	if path == "/" or path:match("^/[A-Za-z]:$") then
		local stat = stat(url)
		return File { url = url, stat = stat, lstat = stat }
	end

	local source = target(url)
	if not source then
		return nil, Error.fs { kind = "NotFound", message = "Invalid place location" }
	end

	local file, err = fs.file(source)
	return file and File { url = url, stat = file.stat, lstat = file.lstat, link_to = file.link_to }, err
end

function M:setup()
	if ya.target_os() ~= "windows" then
		return
	end

	local function enter(args)
		local h = cx.tab.current.hovered
		if not h or not h.stat.is_dir or h.url.spec.scheme ~= "place" then
			return args
		end

		local url = target(h.url)
		if not url then
			return args
		end

		ya.emit("cd", { url, raw = true, tab = cx.tab.id })
	end

	local function leave(args)
		local cwd = cx.tab.current.cwd
		local drive = tostring(cwd.physical):match("^([A-Za-z]:)[/\\]$")
		if not drive then
			return args
		end

		local hovered = cx.tab.current.hovered
		local parent = hovered and hovered.url.parent
		if parent and parent ~= cwd then
			return args
		end

		ya.emit("reveal", { Url("place://host//"):join(drive:upper()), raw = true, tab = cx.tab.id })
	end

	for _, source in ipairs { "key", "ind", "emit", "relay" } do
		ps.sub(source .. "-enter", enter)
		ps.sub(source .. "-leave", leave)
	end
end

function M:provide(job)
	local op = job.op
	if op == "Capabilities" then
		return { reroute = 2 }
	elseif op == "File" or op == "Revalidate" then
		return file(job.url or job.file.url)
	elseif op == "Metadata" or op == "SymlinkMetadata" then
		return stat(job.url, op == "Metadata")
	elseif op == "Reroute" then
		local source = target(job.url)
		if source then
			return fs.file(source)
		end
	elseif op == "ReadDir" and not job.url.name then
		return ya.co(function()
			for _, p in ipairs(fs.partitions()) do
				local drive = p.dist and tostring(p.dist):match("^([A-Za-z]:)[/\\]$")
				if drive then
					coroutine.yield(file(job.url:join(drive:upper())))
				end
			end
		end)
	elseif op == "ReadDir" and target(job.url) then
		return ya.co(function()
			local files, err = fs.read_dir(target(job.url), { resolve = true })
			if not files then
				return nil, err
			end
			for _, file in ipairs(files) do
				coroutine.yield(File {
					url = job.url:join(file.name),
					stat = file.stat,
					lstat = file.lstat,
					link_to = file.link_to,
				})
			end
		end)
	end

	return false, Error.fs { kind = "Unsupported", message = "place:// does not support " .. op }
end

return M
