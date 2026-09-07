local M = {}
local search = require("search")

local function warn(content) ya.notify { title = "Fd", content = tostring(content), timeout = 5, level = "warn" } end

local state = ya.sync(function() return cx.active.current.cwd, cx.active.pref.show_hidden end)

function M:entry(job)
	local root, hidden = state()
	if job.args["in"] then
		local ok, url = pcall(Url, job.args["in"])
		root = ok and url or root
	end

	local source = root.physical
	if not source.is_absolute then
		return warn("Invalid search root")
	elseif not source.spec.is_regular then
		return warn("Only local filesystem searches are supported")
	end

	local subject, event = ya.input {
		name = "fd",
		title = th.fd.title,
		value = job.args[1] or "",
		history = "shared",
		pos = th.fd.position,
	}
	if event ~= 1 then
		return
	end

	local args = job.args.args or ""
	if type(args) ~= "string" then
		return warn("Invalid search arguments")
	end

	local args, err = ya.shell.split(args)
	if not args then
		return warn(err)
	end

	ya.emit("cd", {
		Url {
			source,
			scheme = "fd",
			domain = "default",
			data = { subject, a = args, h = hidden },
		},
		raw = true,
	})
end

function M:provide(job)
	local op = job.op
	if op == "Capabilities" then
		return { file = true, read_dir = true, revalidate = true }
	elseif op == "File" then
		return search.file(job.url)
	elseif op == "Revalidate" then
		return search.file(job.file.url)
	elseif op ~= "ReadDir" then
		return false, Err("Unsupported search operation: %s", op)
	end

	return ya.co(function()
		local source = job.url.physical
		if not source.spec.is_regular then
			return nil, Err("Only local filesystem searches are supported")
		end

		local child, err = M.spawn("fd", source, job.url.spec.data)
		if not child then
			child, err = M.spawn("fdfind", source, job.url.spec.data)
		end
		if not child then
			return nil, Err("Failed to start `fd`, error: %s", err)
		end

		while true do
			local line, event = child:read_line()
			if event ~= 0 then
				break
			end

			local path = search.relative_url(job.url, line)
			if path then
				local file = search.file(path)
				if file then
					coroutine.yield { file = file, cha = file.cha }
				end
			end
		end

		child:wait()
	end)
end

function M.spawn(program, source, data)
	local args = data.a or {}
	local subject = data[1]
	if not M.regex_disabled(args) then
		subject = ya.regex.normalize(subject) or subject
	end

	return Command(program)
		:arg({
			"--base-directory",
			tostring(source),
			"--regex",
			data.h and "--hidden" or "--no-hidden",
		})
		:arg(args)
		:arg("--")
		:arg(subject)
		:stdout(Command.PIPED)
		:spawn()
end

function M.regex_disabled(args)
	local glob = false
	for _, arg in ipairs(args) do
		if arg == "--exact" or arg == "--fixed-strings" then
			return true
		elseif arg == "--glob" then
			glob = true
		elseif arg == "--regex" then
			glob = false
		elseif arg:sub(1, 2) ~= "--" and arg:sub(1, 1) == "-" then
			local flags = arg:sub(2)
			if flags:find("F", 1, true) then
				return true
			end
			glob = glob or flags:find("g", 1, true) ~= nil
		end
	end
	return glob
end

return M
