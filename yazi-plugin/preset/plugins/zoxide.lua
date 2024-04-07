local state = ya.sync(function(st)
	return {
		cwd = tostring(cx.active.current.cwd),
		empty = st.empty,
	}
end)

local set_state = ya.sync(function(st, empty) st.empty = empty end)

local function notify(s, ...)
	ya.notify { title = "Zoxide", content = string.format(s, ...), timeout = 5, level = "error" }
end

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
					confirm = true,
					"zoxide add " .. ya.quote(tostring(cx.active.current.cwd)),
				})
			end
		)
	end
end

local function entry()
	local st = state()
	if st.empty == true then
		return notify("No directory history in the database, check out the `zoxide` docs to set it up.")
	elseif st.empty == nil and head(st.cwd) < 2 then
		set_state(true)
		return notify("No directory history in the database, check out the `zoxide` docs to set it up.")
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
		return notify("Spawn `zoxide` failed with error code %s. Do you have it installed?", err)
	end

	local output, err = child:wait_with_output()
	if not output then
		return notify("`zoxide` exited with error code %s", err)
	end
	ya.manager_emit("cd", { output.stdout:gsub("\n$", "") })
end

return { setup = setup, entry = entry }
