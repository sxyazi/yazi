local M = {}

function M:setup()
	ps.sub_remote("dds-cd", function(url) ya.manager_emit("cd", { url }) end)
end

return M
