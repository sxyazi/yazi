local M = {}

local function top(url)
	while url.parent and url.parent.name do
		url = url.parent
	end
	return url.name and url or nil
end

local function node(url)
	local top = top(url)
	if not top then
		return
	end
	return {
		key = url.spec.domain,
		top = top.spec.domain,
		rel = url:strip_prefix(top) or Path.os(""),
	}
end

local function absolute(url)
	if url.is_absolute then
		return fs.clean_url(url)
	end

	local cwd, err = fs.cwd()
	if not cwd then
		return nil, err
	end

	local root = cwd.path
	while root.parent do
		root = root.parent
	end
	return fs.clean_url(url:join(root:join(url.path)))
end

local function file(url, entry)
	entry.url = url
	return File(entry)
end

local function files(parent, entries)
	for i, entry in ipairs(entries) do
		local url = parent:join(Path.os(entry.name)):into_domain(entry.key)
		entries[i] = file(url, entry)
	end
	return entries
end

local function notify(action, err)
	ya.notify {
		title = "Trash",
		content = string.format("Failed to %s: %s", action, err),
		level = "error",
		timeout = 10,
	}
end

function M:setup()
	ps.sub_remote("trash-restore", function(args)
		ya.async(function()
			local nodes = {}
			for i, arg in ipairs(args) do
				nodes[i] = node(Url(arg))
				if not nodes[i] then
					return notify("restore", "Cannot restore the trash root")
				end
			end

			local ok, err = fs.trash.restore(nodes)
			if ok then
				ya.emit("escape", { select = true })
				ya.emit("refresh", {})
			else
				notify("restore", err)
			end
		end)
	end)

	ps.sub_remote("trash-empty", function()
		ya.async(function()
			local confirmed = ya.confirm {
				pos = { "center", w = 60, h = 10 },
				title = ui.Line("Empty trash?"):style(th.confirm.title),
				body = ui.Text("All items in the trash will be permanently deleted."),
			}
			if not confirmed then
				return
			end

			local ok, err = fs.trash.empty()
			if ok then
				ya.emit("refresh", {})
			else
				notify("empty", err)
			end
		end)
	end)
end

function M:entry()
	local url, err = absolute(Url("trash:///@/."))
	if url then
		ya.emit("cd", { url })
	else
		notify("open", err)
	end
end

function M:provide(job)
	local op = job.op
	if op == "Absolute" or op == "Canonicalize" then
		return absolute(job.url)
	elseif op == "Casefold" then
		return job.url
	elseif op == "Metadata" or op == "SymlinkMetadata" then
		local n = node(job.url)
		if n then
			return fs.trash.metadata(n, op == "Metadata")
		else
			return Cha { mode = tonumber("40700", 8) }
		end
	elseif op == "ReadDir" then
		local entries, err = fs.trash.list(node(job.url))
		if entries then
			return files(job.url, entries)
		else
			return nil, err
		end
	elseif op == "Revalidate" then
		return fs.trash.revalidate(node(job.file.url), job.file)
	elseif op == "File" then
		local n = node(job.url)
		if not n then
			return nil, Error.custom("Cannot construct a file for the trash root")
		end
		local entry, err = fs.trash.entry(n)
		if entry then
			return file(job.url, entry)
		else
			return nil, err
		end
	elseif op == "RemoveFile" then
		return fs.trash.remove("file", node(job.url))
	elseif op == "RemoveDir" then
		return fs.trash.remove("dir", node(job.url))
	end

	return false, Error.custom("Unsupported trash operation: " .. op)
end

return M
