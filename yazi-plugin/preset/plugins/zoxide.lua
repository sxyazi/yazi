local M = {}

local state = ya.sync(function(st)
	return {
		cwd = tostring(cx.active.current.cwd),
		empty = st.empty,
	}
end)

local set_state = ya.sync(function(st, empty) st.empty = empty end)

-- Bump the entry token and record the directory, so stale `cd` handlers (from
-- directories we've already moved away from) can tell they're no longer current.
local next_entry = ya.sync(function(st, dir)
	st.entry_token = (st.entry_token or 0) + 1
	st.entry_dir = dir
	return st.entry_token
end)

local get_entry = ya.sync(function(st) return st.entry_token, st.entry_dir end)

-- Rate-limit adding the same directory again until `interval` seconds pass.
local try_add = ya.sync(function(st, dir)
	local now = ya.time()
	local prev = st.last[dir]
	if prev and st.interval > 0 and now - prev < st.interval then
		return false
	end
	st.last[dir] = now
	return true
end)

local function display(dir)
	dir = dir:gsub("\\", "/")
	local home = os.getenv("HOME") or os.getenv("USERPROFILE")
	if home then
		home = home:gsub("\\", "/"):gsub("/+$", "")
		local lower_dir, lower_home = dir:lower(), home:lower()
		if lower_dir:sub(1, #lower_home) == lower_home then
			local rest = dir:sub(#lower_home + 1)
			if rest == "" then
				return "~"
			elseif rest:sub(1, 1) == "/" then
				return "~" .. rest
			end
		end
	end
	return dir
end

function M:setup(opts)
	opts = opts or {}

	if opts.update_db then
		M.last = M.last or {}
		M.interval = opts.interval or 0

		local delay = opts.delay or 0
		local notify_time = opts.notify_time or 0

		ps.sub("cd", function()
			local cwd = cx.active.current.cwd
			local dir = tostring(cwd)

			local token = next_entry(dir)

			ya.async(function()
				ya.sleep(delay)
				local cur_token, cur_dir = get_entry()
				if cur_token ~= token or cur_dir ~= dir then
					return
				end
				if not try_add(dir) then
					return
				end
				Command("zoxide"):arg({ "add", dir }):status()
				if notify_time > 0 then
					ya.notify { title = "Zoxide", content = "Added " .. display(dir), timeout = notify_time }
				end
			end)
		end)
	end
end

function M:entry()
	local st = state()
	if st.empty == nil then
		st.empty = M.is_empty(st.cwd)
		set_state(st.empty)
	end

	if st.empty then
		return ya.notify {
			title = "Zoxide",
			content = "No directory history found, check Zoxide's doc to set it up and restart Yazi.",
			timeout = 5,
			level = "error",
		}
	end

	local permit = ui.hide()
	local target, err = M.run_with(st.cwd)
	permit:drop()

	if not target then
		ya.notify { title = "Zoxide", content = tostring(err), timeout = 5, level = "error" }
	elseif target ~= "" then
		ya.emit("cd", { target, raw = true })
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
