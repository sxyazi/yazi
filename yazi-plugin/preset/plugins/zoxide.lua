local state = ya.sync(function(st)
	return {
		cwd = tostring(cx.active.current.cwd),
		empty = st.empty,
	}
end)

local set_state = ya.sync(function(st, empty) st.empty = empty end)

local function fail(s, ...) ya.notify { title = "Zoxide", content = s:format(...), timeout = 5, level = "error" } end

local function head(cwd)
	local child = Command("zoxide"):args({ "query", "-l" }):stdout(Command.PIPED):spawn()
	if not child then
		return 0
	end

	local n = 0
	repeat
		local next, event = child:read_line()
		if event ~= 0 then
			break
		elseif cwd ~= next:gsub("\n$", "") then
			n = n + 1
		end
	until n >= 2

	child:start_kill()
	return n
end

local function setup(_, opts)
	opts = opts or {}

	if opts.update_db then
		ps.sub(
			"cd",
			function()
				ya.manager_emit("shell", {
					orphan = true,
					confirm = true,
					"zoxide add " .. ya.quote(tostring(cx.active.current.cwd)),
				})
			end
		)
	end
end

local function entry()
	local st = state()
	if st.empty == nil then
		st.empty = head(st.cwd) < 2
		set_state(st.empty)
	end

	if st.empty then
		return fail("No directory history in the database, check out the `zoxide` docs to set it up.")
	end

	local _permit = ya.hide()
	local child, err = Command("zoxide")
		:args({ "query", "-i", "--exclude" })
		:arg(st.cwd)
		:stdin(Command.INHERIT)
		:stdout(Command.PIPED)
		:stderr(Command.INHERIT)
		:spawn()

	if not child then
		return fail("Spawn `zoxide` failed with error code %s. Do you have it installed?", err)
	end

	local output, err = child:wait_with_output()
	if not output then
		return fail("Cannot read `zoxide` output, error code %s", err)
	elseif not output.status.success and output.status.code ~= 130 then
		return fail("`zoxide` exited with error code %s", output.status.code)
	end

	local target = output.stdout:gsub("\n$", "")
	if target ~= "" then
		ya.manager_emit("cd", { target })
	end
end

return { setup = setup, entry = entry }
