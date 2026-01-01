-- stylua: ignore
local TYPE_PATS = { "text", "image", "video", "application", "audio", "font", "inode", "message", "model", "vector", "biosig", "chemical", "rinex", "x%-epoc" }

local M = {}

function M:fetch(job)
	local urls, paths = {}, {}
	for i, file in ipairs(job.files) do
		if file.cache then
			urls[i], paths[i] = tostring(file.url), tostring(file.cache)
		else
			paths[i] = tostring(file.path)
		end
	end

	local child, err = M.spawn_file1(paths)
	if not child then
		M.placeholder(err, urls, paths)
		return true, err
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

		match, ignore = M.match_mimetype(line)
		if match then
			updates[urls[i] or paths[i]], state[i], i = match, true, i + 1
			flush(false)
		elseif not ignore then
			state[i], i = false, i + 1
		end
		::continue::
	until i > #paths

	flush(true)
	return state
end

function M.match_mimetype(line)
	for _, pat in ipairs(TYPE_PATS) do
		local typ, sub = line:match(string.format("(%s/)([+-.a-zA-Z0-9]+)%%s+$", pat))
		if not sub then
		elseif line:find(typ .. sub, 1, true) == 1 then
			return typ:gsub("^x%-", "", 1) .. sub:gsub("^x%-", "", 1):gsub("^vnd%.", "", 1)
		else
			return nil, true
		end
	end
end

function M.file1_bin() return os.getenv("YAZI_FILE_ONE") or "file" end

function M.spawn_file1(paths)
	local bin = M.file1_bin()
	local windows = ya.target_family() == "windows"

	local cmd = Command(bin):arg({ "-bL", "--mime-type" }):stdout(Command.PIPED)
	if windows then
		cmd:arg({ "-f", "-" }):stdin(Command.PIPED)
	else
		cmd:arg("--"):arg(paths)
	end

	local child, err = cmd:spawn()
	if not child then
		local e = Error.fs {
			kind = err.kind or "Other",
			code = err.code,
			message = string.format("Failed to start `%s`, error: %s", bin, err),
		}
		return nil, e
	elseif windows then
		child:write_all(table.concat(paths, "\n"))
		child:flush()
		ya.drop(child:take_stdin())
	end

	return child
end

function M.placeholder(err, urls, paths)
	if err.kind ~= "NotFound" then
		return
	end

	local updates = {}
	for i = 1, #paths do
		updates[urls[i] or paths[i]] = "null/file1-not-found"
	end

	ya.emit("update_mimes", { updates = updates })
end

return M
