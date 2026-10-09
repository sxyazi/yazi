local M = {}

local state = ya.sync(function(st) return tostring(cx.tab.current.cwd), st.empty end)

local set_state = ya.sync(function(st, empty) st.empty = empty end)

function M:setup(opts)
	opts = opts or {}

	if opts.update_db then
		ps.sub("cd", function()
			local cwd = cx.tab.current.cwd
			ya.async(function() Command("zoxide"):arg({ "add", tostring(cwd) }):status() end)
		end)
	end
end

function M:entry(job)
	local cwd, empty = state()
	if empty == nil then
		empty = M.is_empty(cwd)
		set_state(empty)
	end

	if empty then
		return ya.notify {
			title = "Zoxide",
			content = "No directory history found, check Zoxide's doc to set it up and restart Yazi.",
			timeout = 5,
			level = "error",
		}
	end

	local permit = ui.hide()
	local target, err = M.run_with(cwd)
	permit:drop()

	if not target then
		ya.notify { title = "Zoxide", content = tostring(err), timeout = 5, level = "error" }
	elseif target ~= "" then
		ya.emit("cd", { target, raw = true, tab = job.tab })
	end
end

function M.options()
	-- https://github.com/ajeetdsouza/zoxide/blob/main/src/cmd/query.rs#L92
	local default = {
		-- Search mode
		"--exact",
		-- Search result
		"--no-sort",
		-- Interface
		"--bind=ctrl-z:ignore,btab:up,tab:down",
		"--cycle",
		"--keep-right",
		-- Layout
		"--layout=reverse",
		"--height=100%",
		"--border",
		"--scrollbar=▌",
		"--info=inline",
		-- Display
		"--tabstop=1",
		-- Scripting
		"--exit-0",
	}

	if ya.target_family() == "unix" then
		default[#default + 1] = "--preview-window=down,30%,sharp"
		if ya.target_os() == "linux" then
			default[#default + 1] = [[--preview='\command -p ls -Cp --color=always --group-directories-first {2..}']]
		else
			default[#default + 1] = [[--preview='\command -p ls -Cp {2..}']]
		end
	end

	return (os.getenv("FZF_DEFAULT_OPTS") or "")
		.. " "
		.. table.concat(default, " ")
		.. " "
		.. (os.getenv("YAZI_ZOXIDE_OPTS") or "")
end

---@param cwd string
---@return boolean
function M.is_empty(cwd)
	local child = Command("zoxide"):arg({ "query", "-l", "--exclude", cwd }):stdout(Command.PIPED):spawn()
	if not child then
		return true
	end

	local first = child:read_line()
	child:start_kill()
	return not first
end

---@param cwd string
---@return string?, Error?
function M.run_with(cwd)
	local child, err = Command("zoxide")
		:arg({ "query", "-i", "--exclude", cwd })
		:env("SHELL", "sh")
		:env("CLICOLOR", 1)
		:env("CLICOLOR_FORCE", 1)
		:env("_ZO_FZF_OPTS", M.options())
		:stdin(Command.INHERIT)
		:stdout(Command.PIPED)
		:stderr(Command.PIPED)
		:spawn()

	if not child then
		return nil, Err("Failed to start `zoxide`, error: %s", err)
	end

	local output, err = child:wait_with_output()
	if not output then
		return nil, Err("Cannot read `zoxide` output, error: %s", err)
	elseif not output.status.success and output.status.code ~= 130 then
		return nil, Err("`zoxide` exited with code %s: %s", output.status.code, output.stderr:gsub("^zoxide:%s*", ""))
	end
	return output.stdout:gsub("\n$", ""), nil
end

return M
