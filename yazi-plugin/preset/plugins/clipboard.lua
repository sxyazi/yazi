local M = {}

function M.mime_ext(mime)
	return ({ [""] = "txt", ["image/jpeg"] = "jpg", ["image/svg+xml"] = "svg", ["text/plain"] = "txt" })[mime]
		or mime:match("/([%w]+)$")
		or "bin"
end

function M.read_unsolicited(mime, event)
	rt.tty:queue("ReadClipboard", {
		mimes = { mime },
		pw = event.pw,
		name = "Paste event",
		primary = event.primary,
	})
	rt.tty:flush()
end

function M.copy_uri_list(list)
	cx.tasks.behavior:reset()
	for line in list:gmatch("[^\r\n]+") do
		if line:sub(1, 7) ~= "file://" then
			goto continue
		end

		local from = Url(ya.percent_decode(line:sub(8)))
		if from.name then
			local to = cx.active.current.cwd:join(from.name)
			ya.async(function() ya.task("copy", { from = from, to = to }):spawn() end)
		end

		::continue::
	end
end

-- TODO: remove
local path = tostring(rt.path.config_dir:join("plugins/clipboard.yazi/main.lua"))
local try_compat = ya.sync(function(state, path)
	if state._compat_loaded then
		return true
	end

	local ok = pcall(dofile, path)
	if ok then
		ya.notify {
			title = "Heads up:",
			content = "`clipboard` is deprecated as a custom plugin name in favor of a built-in one.\n\nRename your `plugins/clipboard.yazi` to a different name, and update your `keymap.toml` to use the new name instead.",
			level = "warn",
			timeout = 20,
		}
	end

	state._compat_loaded = ok
	return ok
end)

function M:entry(job)
	if try_compat(path) then
		return dofile(path):entry(job)
	end
end

return M
