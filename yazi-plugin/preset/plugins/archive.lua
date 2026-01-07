local M = {}

function M:peek(job)
	local limit = job.area.h
	local files, bound, err = nil, nil, nil

	if M.is_compressed_tar(job.file.path) then
		files, bound, err = self.list_files_tar({ "-p", tostring(job.file.path) }, job.skip, limit)
	else
		files, bound, err = self.list_files({ "-p", tostring(job.file.path) }, job.skip, limit)
	end

	if err then
		return ya.preview_widget(job, err)
	elseif job.skip > 0 and bound < job.skip + limit then
		return ya.emit("peek", { math.max(0, bound - limit), only_if = job.file.url, upper_bound = true })
	elseif #files == 0 then
		files = { { path = job.file.url.stem, size = 0, attr = "" } }
	end

	local left, right = {}, {}
	for _, f in ipairs(files) do
		local icon = File({
			url = Url(f.path),
			cha = Cha { mode = tonumber(f.attr:sub(1, 1) == "D" and "40700" or "100644", 8) },
		}):icon()

		if f.size > 0 then
			right[#right + 1] = string.format(" %s ", ya.readable_size(f.size))
		else
			right[#right + 1] = " "
		end

		if icon then
			left[#left + 1] = ui.Span(" " .. icon.text .. " "):style(icon.style)
		else
			left[#left + 1] = " "
		end

		left[#left] = ui.Line {
			left[#left],
			ui.truncate(f.path, {
				rtl = true,
				max = math.max(0, job.area.w - ui.width(left[#left]) - ui.width(right[#right])),
			}),
		}
	end

	ya.preview_widget(job, {
		ui.Text(left):area(job.area),
		ui.Text(right):area(job.area):align(ui.Align.RIGHT),
	})
end

function M:seek(job) require("code"):seek(job) end

function M.spawn_7z(args)
	local last_err = nil
	local try = function(name)
		local stdout = args[1] == "l" and Command.PIPED or Command.NULL
		local child, err = Command(name):arg(args):stdout(stdout):stderr(Command.PIPED):spawn()
		if not child then
			last_err = err
		end
		return child
	end

	local child = try("7zz") or try("7z")
	if not child then
		return ya.err("Failed to start either `7zz` or `7z`, error: " .. last_err)
	end
	return child, last_err
end

-- Spawn a 7z instance which pipes a "7z {argsX}" into a "7z {argsL}"
-- Used for previewing tar.* archives, by doing "7z x -so .. | 7z l -si .."
function M.spawn_7z_piped(argsX, argsL)
	local last_err = nil
	local try = function(name)
		local childX, err = Command(name):arg(argsX):stdout(Command.PIPED):stderr(Command.PIPED):spawn()
		if not childX then
			last_err = err
			return childX, nil
		end
		local childL, err =
			Command(name):arg(argsL):stdin(childX:take_stdout()):stdout(Command.PIPED):stderr(Command.PIPED):spawn()
		if not childL then
			last_err = err
		end
		return childX, childL
	end

	local childX, childL = try("7zz")
	if not childX or not childL then
		childX, childL = try("7z")
	end
	if not childX or not childL then
		return ya.err("Failed to start either `7zz` or `7z`, error: " .. last_err)
	end
	return childX, childL, last_err
end

-- Parse the output of a "7z l -slt" command. The caller is responsible for killing the child process right after the execution of this function
---@param child Child
---@param skip integer
---@param limit integer
---@return table files
---@return integer bound
---@return Error? err
function M.parse_7z_list(child, skip, limit)
	local i, files, err = 0, { { path = "", size = 0, attr = "" } }, nil
	local key, value, stderr = "", "", {}
	repeat
		local next, event = child:read_line()
		if event == 1 and M.is_encrypted(next) then
			err = Err("File list in this archive is encrypted")
			break
		elseif event == 1 then
			stderr[#stderr + 1] = next
			goto continue
		elseif event ~= 0 then
			break
		end

		if next == "\n" or next == "\r\n" then
			i = i + 1
			if files[#files].path ~= "" then
				files[#files + 1] = { path = "", size = 0, attr = "" }
			end
			goto continue
		elseif i < skip then
			goto continue
		end

		key, value = next:match("^(%u%l+) = (.-)[\r\n]+")
		if key == "Path" then
			files[#files].path = value
		elseif key == "Size" then
			files[#files].size = tonumber(value) or 0
		elseif key == "Attributes" then
			files[#files].attr = value
		end

		::continue::
	until i >= skip + limit

	if files[#files].path == "" then
		files[#files] = nil
	end
	if #stderr ~= 0 then
		err = Err("7-zip errored out while listing files, stderr: %s", table.concat(stderr, "\n"))
	end
	return files, i, err
end

---List files in an archive (non-tar)
---@param args table
---@param skip integer
---@param limit integer
---@return table files
---@return integer bound
---@return Error? err
function M.list_files(args, skip, limit)
	local child = M.spawn_7z { "l", "-ba", "-slt", "-sccUTF-8", table.unpack(args) }
	if not child then
		return {}, 0, Err("Failed to start either `7zz` or `7z`. Do you have 7-zip installed?")
	end

	local files, bound, err = M.parse_7z_list(child, skip, limit)
	child:start_kill()

	return files, bound, err
end

---List files in a tar.* archive
---@param args table
---@param skip integer
---@param limit integer
---@return table files
---@return integer bound
---@return Error? err
function M.list_files_tar(args, skip, limit)
	local childX, childL = M.spawn_7z_piped(
		{ "x", "-so", table.unpack(args) },
		{ "l", "-ba", "-slt", "-ttar", "-sccUTF-8", "-si" }
	)
	if not childX or not childL then
		return {}, 0, Err("Failed to start either `7zz` or `7z`. Do you have 7-zip installed?")
	end

	local files, bound, err = M.parse_7z_list(childL, skip, limit)
	childL:start_kill()
	childX:start_kill()

	return files, bound, err
end

---List metadata of an archive
---@param args table
---@return string|nil type
---@return integer code
---  0: success
---  1: failed to spawn
---  2: wrong password
---  3: partial success
function M.list_meta(args)
	local child = M.spawn_7z { "l", "-slt", "-sccUTF-8", table.unpack(args) }
	if not child then
		return nil, 1
	end

	local i, head = 0, ""
	local typ, code = nil, 0
	while i < 500 do
		i = i + 1

		local next, event = child:read_line()
		if event == 1 and M.is_encrypted(next) then
			code = 2
			break
		elseif event == 1 then
			code = 3
		elseif event == 0 then
			head = head .. next
		else
			break
		end

		typ = head:gmatch("--[\r\n]+Path = .-[\r\n]+Type = (.-)[\r\n]+")()
		if typ then
			break
		end
	end

	child:start_kill()
	return typ ~= "" and typ or nil, code
end

function M.is_encrypted(s) return s:find(" Wrong password", 1, true) end

function M.is_tar(path) return M.list_meta { "-p", tostring(path) } == "tar" end

function M.is_compressed_tar(path)
	-- Warning: doing -slt will *not* print the .tar file, it will only print the .tar.* file
	-- doing -ba will print the .tar file in the listing, as the first line
	local child = M.spawn_7z { "l", "-ba", "-sccUTF-8", "-p", tostring(path) }
	if not child then
		return false
	end

	local names = {}
	while #names < 2 do
		local next, event = child:read_line()
		if event == 0 then
			local name = next:sub(20):match("[^ ]+ +[^ ]+ +[^ ]+ +([^\r\n]+)")
			if name then
				names[#names + 1] = name
			end
		elseif event ~= 1 then
			break
		end
	end

	child:start_kill()
	return #names == 1 and names[1]:find(".+%.tar$") ~= nil
end

return M
