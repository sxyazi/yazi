local M = {}

local state = ya.sync(function()
	local selected = {}
	for _, url in pairs(cx.active.selected) do
		selected[#selected + 1] = url
	end
	return cx.active.current.cwd, selected
end)

function M:entry(job)
	ya.emit("escape", { visual = true })

	local cwd, selected = state()
	if cwd.scheme.is_virtual then
		return ya.notify { title = "Fzf", content = "Not supported under virtual filesystems", timeout = 5, level = "warn" }
	end

	local default_cmd = M.parse_args(job and job.args or {})
	if default_cmd == "" then
		return ya.notify { title = "Fzf", content = "Missing string after --fzf-command", timeout = 5, level = "error" }
	end
	local permit = ui.hide()
	local output, err = M.run_with(cwd, selected, default_cmd)

	permit:drop()
	if not output then
		return ya.notify { title = "Fzf", content = tostring(err), timeout = 5, level = "error" }
	end

	local urls = M.split_urls(cwd, output)
	if #urls == 1 then
		local cha = #selected == 0 and fs.cha(urls[1])
		ya.emit(cha and cha.is_dir and "cd" or "reveal", { urls[1], raw = true })
	elseif #urls > 1 then
		urls.state = #selected > 0 and "off" or "on"
		ya.emit("toggle_all", urls)
	end
end

---@param cwd Url
---@param selected Url[]
---@return string?, Error?
function M.run_with(cwd, selected, default_cmd)
	local input = nil
	local source = "stdin"
	if #selected > 0 then
		source = "selection"
		input = ""
		for _, u in ipairs(selected) do
			input = input .. string.format("%s\n", u)
		end
	end

	local cmd = Command("fzf"):arg("-m")
	if default_cmd and #selected == 0 then
		cmd:env("FZF_DEFAULT_COMMAND", default_cmd)
	end
	local child, err = cmd
		:cwd(tostring(cwd))
		:stdin(input and Command.PIPED or Command.INHERIT)
		:stdout(Command.PIPED)
		:spawn()

	if not child then
		return nil, Err("Failed to start `fzf`, error: %s", err)
	end

	if input then
		child:write_all(input)
		child:flush()
	end

	local output, err = child:wait_with_output()
	if not output then
		return nil, Err("Cannot read `fzf` output, error: %s", err)
	elseif not output.status.success and output.status.code ~= 130 then
		return nil, Err("`fzf` exited with error code %s", output.status.code)
	end
	return output.stdout, nil
end

function M.parse_args(args)
	if not args then
		return nil
	end

	local v = args.fzf_command or args["fzf-command"]
	if type(v) == "string" then
		return v
	end

	if v ~= nil then
		return ""
	end

	return nil
end

function M.split_urls(cwd, output)
	local t = {}
	for line in output:gmatch("[^\r\n]+") do
		local u = Url(line)
		if u.is_absolute then
			t[#t + 1] = u
		else
			t[#t + 1] = cwd:join(u)
		end
	end
	return t
end

return M
