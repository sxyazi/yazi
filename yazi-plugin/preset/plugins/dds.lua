local M = {}

function M:setup()
	-- TODO: remove this
	local b = false
	ps.sub_remote("dds-cd", function(url)
		if not b then
			b = true
			ya.notify {
				title = "Deprecated DDS Event",
				content = "The `dds-cd` event is deprecated, please use `ya emit cd /your/path` instead of `ya pub dds-cd --str /your/path`\n\nSee #1979 for details: https://github.com/sxyazi/yazi/pull/1979",
				timeout = 20,
				level = "warn",
			}
		end
		ya.manager_emit("cd", { url })
	end)

	ps.sub_remote("dds-emit", function(cmd) ya.manager_emit(cmd[1], cmd[2]) end)
end

return M
