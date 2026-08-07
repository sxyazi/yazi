-- stylua: ignore
local TYPE_PATS = { "text", "image", "video", "application", "audio", "font", "inode", "message", "model", "vector", "biosig", "chemical", "rinex", "x%-epoc" }

local M = {}

function M:fetch(job)
	return ya.co(function()
		local paths, updates = {}, {}
		for i, file in ipairs(job.files) do
			paths[i] = tostring(file.path)
		end

		local flush = ya.throttle(0.3, function()
			if next(updates) then
				ya.emit("update_mimes", { updates = updates })
				updates = {}
			end
		end)

		local child, err = M.spawn_file1(paths)
		if not child then
			return M.placeholder(err, job.files)
		end

		local i, match, ignore = 1, nil, nil
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
				if coroutine.yield(job.files[i], match) then
					updates[job.files[i].url] = match
					flush()
				end
				i = i + 1
			elseif not ignore then
				coroutine.yield(job.files[i], Err("Failed to determine MIME type for `%s`", job.files[i].url))
				i = i + 1
			end
			::continue::
		until i > #paths

		for j = i, #paths do
			coroutine.yield(job.files[j], Err("Failed to read `file` output"))
		end
		flush(true)
	end)
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

function M.placeholder(err, files)
	local mime, updates = "null/file1-not-found", {}
	for _, file in ipairs(files) do
		if err.kind ~= "NotFound" then
			coroutine.yield(file, Error(err))
		elseif coroutine.yield(file, mime) then
			updates[file.url] = mime
		end
	end
	return require("mime.dir").commit(updates)
end

return M
