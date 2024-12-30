local state = ya.sync(function(st)
	return {
		cwd = tostring(cx.active.current.cwd),
		empty = st.empty,
	}
end)

local set_state = ya.sync(function(st, empty) st.empty = empty end)

local function fail(s, ...) ya.notify { title = "Zoxide", content = s:format(...), timeout = 5, level = "error" } end

local function opts()
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

local function empty(cwd)
	local child = Command("zoxide"):args({ "query", "-l", "--exclude" }):arg(cwd):stdout(Command.PIPED):spawn()
	if not child then
		return true
	end

	local first = child:read_line()
	child:start_kill()
	return not first
end

local function setup(_, options)
	options = options or {}

	if options.update_db then
		ps.sub(
			"cd",
			function()
				ya.manager_emit("shell", {
					cwd = fs.cwd(),
					orphan = true,
					"zoxide add " .. ya.quote(tostring(cx.active.current.cwd)),
				})
			end
		)
	end
end

local function entry()
	local st = state()
	if st.empty == nil then
		st.empty = empty(st.cwd)
		set_state(st.empty)
	end

	if st.empty then
		return fail("No directory history found, check Zoxide's doc to set it up and restart Yazi.")
	end

	local _permit = ya.hide()

	local fzf, _err = Command("fzf"):arg("--version"):stdout(Command.PIPED):stderr(Command.PIPED):output()

	if not fzf then
		return fail(string.format("%s\n%s", "`fzf` is required for the `zoxide` plugin.", "Please install `fzf`."))
	end

	local child, err = Command("zoxide")
		:args({ "query", "-i", "--exclude" })
		:arg(st.cwd)
		:env("SHELL", "sh")
		:env("CLICOLOR", "1")
		:env("CLICOLOR_FORCE", "1")
		:env("_ZO_FZF_OPTS", opts())
		:stdin(Command.INHERIT)
		:stdout(Command.PIPED)
		:stderr(Command.INHERIT)
		:spawn()

	if not child then
		return fail("Failed to start `zoxide`, error: " .. err)
	end

	local output, error = child:wait_with_output()
	if not output then
		return fail("Cannot read `zoxide` output, error: " .. error)
	elseif not output.status.success and output.status.code ~= 130 then
		return fail("`zoxide` exited with error code %s", output.status.code)
	end

	local target = output.stdout:gsub("\n$", "")
	if target ~= "" then
		ya.manager_emit("cd", { target })
	end
end

return { setup = setup, entry = entry }
