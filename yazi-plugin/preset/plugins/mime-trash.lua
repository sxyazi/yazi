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
		local file, result = next()
		while file do
			local mime = type(result[1]) == "string" and "trash/" .. result[1]
			if mime then
				result[1] = mime
			end

			if coroutine.yield(file, result) and not file.cha.is_dummy then
				updates[file.url] = mime
				flush()
			end
			file, result = next()
		end
		flush(true)
	end)
end

return M
