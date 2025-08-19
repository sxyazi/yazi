-- stylua: ignore
local TYPE_PATS = { "text", "image", "video", "application", "audio", "font", "inode", "message", "model", "vector", "biosig", "chemical", "rinex", "x%-epoc" }

local M = {}

local function match_mimetype(s)
	for _, pat in ipairs(TYPE_PATS) do
		local typ, sub = s:match(string.format("(%s/)([+-.a-z0-9]+)%%s+$", pat))
		if not sub then
		elseif s:find(typ .. sub, 1, true) == 1 then
			return typ:gsub("^x%-", "", 1) .. sub:gsub("^x%-", "", 1):gsub("^vnd%.", "", 1)
		else
			return nil, true
		end
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

	local i, state, match, ignore = 1, {}, nil, nil
	repeat
		local line, event = child:read_line_with { timeout = 300 }
		if event == 3 then
			flush(true)
			goto continue
		elseif event ~= 0 then
			break
		end

		match, ignore = match_mimetype(line)
		if match then
			updates[urls[i]], state[i], i = match, true, i + 1
			flush(false)
		elseif not ignore then
			state[i], i = false, i + 1
		end
		::continue::
	until i > #urls

	flush(true)
	return state
end

return M
