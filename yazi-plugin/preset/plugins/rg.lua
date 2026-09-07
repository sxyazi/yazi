local M = {}
local search = require("search")

local function warn(content) ya.notify { title = "Rg", content = tostring(content), timeout = 5, level = "warn" } end

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
		name = "rg",
		title = th.rg.title,
		value = job.args[1] or "",
		history = "shared",
		pos = th.rg.position,
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
			scheme = "rg",
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

		local child, err = M.spawn(source, job.url.spec.data)
		if not child then
			return nil, Err("Failed to start `rg`, error: %s", err)
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

function M.spawn(source, data)
	return Command("rg")
		:cwd(tostring(source))
		:arg({ "--color=never", "--files-with-matches", "--smart-case" })
		:arg(data.h and "--hidden" or "--no-hidden")
		:arg(data.a or {})
		:arg("--")
		:arg(data[1])
		:stdout(Command.PIPED)
		:spawn()
end

return M
