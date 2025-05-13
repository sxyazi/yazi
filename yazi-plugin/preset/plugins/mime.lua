local SUPPORTED_TYPES = "application/audio/biosig/chemical/font/image/inode/message/model/rinex/text/vector/video/x-epoc/"

local M = {}

local function match_mimetype(s)
	local type, sub = s:match("^([-a-z]+/)([+-.a-zA-Z0-9]+)%s*$")
	if type and sub and SUPPORTED_TYPES:find(type, 1, true)  then
		return type:gsub("^x%-", "", 1) .. sub:gsub("^x%-", "", 1):gsub("^vnd%.", "", 1)
	end
end

function M:fetch(job)
	local urls = {}
	for _, file in ipairs(job.files) do
		urls[#urls + 1] = tostring(file.url)
	end

	local cmd = os.getenv("YAZI_FILE_ONE") or "file"
	local child, err = Command(cmd):arg({ "-bL", "--mime-type", "--" }):arg(urls):stdout(Command.PIPED):spawn()
	if not child then
		return true, Err("Failed to start `%s`, error: %s", cmd, err)
	end

	local updates, last = {}, ya.time()
	local flush = function(force)
		if not force and ya.time() - last < 0.3 then
			return
		end
		if next(updates) then
			ya.emit("update_mimes", { updates = updates })
			updates, last = {}, ya.time()
		end
	end

	local i, valid, state = 1, nil, {}
	repeat
		local line, event = child:read_line_with { timeout = 300 }
		if event == 3 then
			flush(true)
			goto continue
		elseif event ~= 0 then
			break
		end

		valid = match_mimetype(line)
		if valid then
			updates[urls[i]], state[i] = valid, true
			flush(false)
		else
			state[i] = false
		end

		i = i + 1
		::continue::
	until i > #urls

	flush(true)
	return state
end

return M
