local cwd = ya.sync(function() return tostring(cx.active.current.cwd) end)

local function notify(s, ...) ya.notify { title = "Fzf", content = string.format(s, ...), timeout = 5, level = "error" } end

local function entry()
	local _permit = ya.hide()
	local cwd = cwd()

	local child, err =
		Command("fzf"):cwd(cwd):stdin(Command.INHERIT):stdout(Command.PIPED):stderr(Command.INHERIT):spawn()

	if not child then
		return notify("Spawn `fzf` failed with error code %s. Do you have it installed?", err)
	end

	local output, err = child:wait_with_output()
	if not output then
		return notify("`fzf` exited with error code %s", err)
	end

	local target = output.stdout:gsub("\n$", "")
	ya.manager_emit(target:match("[/\\]$") and "cd" or "reveal", { target })
end

return { entry = entry }
