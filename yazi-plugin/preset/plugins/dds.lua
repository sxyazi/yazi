local M = {}

function M:setup()
	ps.sub_remote("dds-emit", function(cmd) ya.mgr_emit(cmd[1], cmd[2]) end)
end

return M
