local M = {}

function M:fetch(job)
	return ya.co(function()
		local updates = {}
		local flush = ya.throttle(0.3, function()
			if next(updates) then
				ya.emit("update_mimes", { updates = updates })
				updates = {}
			end
		end)

		local next = require("mime.local"):fetch(job)
		local file, value = next()
		while file do
			if type(value) ~= "string" then
				coroutine.yield(file, value)
			elseif coroutine.yield(file, "trash/" .. value) then
				updates[file.url] = "trash/" .. value
				flush()
			end
			file, value = next()
		end
		flush(true)
	end)
end

return M
