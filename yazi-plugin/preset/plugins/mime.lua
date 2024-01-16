local M = {}

function M:preload()
	local urls = {}
	for _, file in ipairs(self.files) do
		urls[#urls + 1] = tostring(file.url)
	end

	local child, code = Command("file"):args({ "-bL", "--mime-type" }):args(urls):stdout(Command.PIPED):spawn()
	if not child then
		ya.err("spawn `file` command returns " .. tostring(code))
		return 0
	end

	local mimes, last = {}, ya.time()
	local flush = function(force)
		if not force and ya.time() - last < 0.1 then
			return
		end
		if next(mimes) then
			ya.manager_emit("update_mimetype", {}, mimes)
			mimes, last = {}, ya.time()
		end
	end

	local i, j = 1, 0
	repeat
		local next, event = child:read_line_with { timeout = 300 }
		if event == 3 then
			flush(true)
			goto continue
		elseif event ~= 0 then
			break
		end

		next = next:gsub("[\r\n]+$", "")
		if ya.mime_valid(next) then
			j, mimes[urls[i]] = j + 1, next
			flush(false)
		end

		i = i + 1
		::continue::
	until i > #urls

	flush(true)
	return j == #urls and 3 or 2
end

return M
