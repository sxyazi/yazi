local M = {}

function M:peek(job)
	local limit = job.area.h
	local files, bound, err = self.list_archive({ "-p", tostring(job.file.path) }, job.skip, limit)

	local first = (#files == 1 and files[1]) or (#files == 0 and M.list_if_only_one(job.file.path))
	if first and M.should_decompress_tar(first) then
		files, bound, err = self.list_compressed_tar({ "-p", tostring(job.file.path) }, job.skip, limit)
	end

	if err then
		return ya.preview_widget(job, err)
	elseif job.skip > 0 and bound < job.skip + limit then
		return ya.emit("peek", { math.max(0, bound - limit), only_if = job.file.url, upper_bound = true })
	elseif #files == 0 then
		files = { { path = job.file.url.stem, size = 0, packed_size = 0, attr = "" } }
	end

	M.prepare_tree(files)

	local left, right = {}, {}
	for _, f in ipairs(files) do
		local icon = File({
			url = Url(f.path),
			cha = Cha { mode = tonumber(f.is_dir and "40700" or "100644", 8) },
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
			string.rep(" │", f.depth),
			left[#left],
			ui.truncate(f.display_name, {
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

-- Spawn a 7z instance which pipes a "7z {src_args}" into a "7z {dst_args}"
-- Used for previewing compressed tarballs, by doing "7z x -so .. | 7z l -si .."
function M.spawn_7z_piped(src_args, dst_args)
	local last_err = nil
	local try = function(name)
		local src, err = Command(name):arg(src_args):stdout(Command.PIPED):stderr(Command.PIPED):spawn()
		if not src then
			last_err = err
			return src
		end
		local dst, err =
			Command(name):arg(dst_args):stdin(src:take_stdout()):stdout(Command.PIPED):stderr(Command.PIPED):spawn()
		if not dst then
			last_err = err
		end
		return src, dst
	end

	local src, dst = try("7zz")
	if not src then
		src, dst = try("7z")
	end
	if not dst then
		return ya.err("Failed to start either `7zz` or `7z`, error: " .. last_err)
	end
	return src, dst, last_err
end

---List files in an archive
---@param args table
---@param skip integer
---@param limit integer
---@return table files
---@return integer bound
---@return Error? err
function M.list_archive(args, skip, limit)
	local child = M.spawn_7z { "l", "-ba", "-slt", "-sccUTF-8", table.unpack(args) }
	if not child then
		return {}, 0, Err("Failed to start either `7zz` or `7z`. Do you have 7-zip installed?")
	end

	local files, bound, err = M.parse_7z_slt(child, skip, limit)
	child:start_kill()

	return files, bound, err
end

---List files in a compressed tarball
---@param args table
---@param skip integer
---@param limit integer
---@return table files
---@return integer bound
---@return Error? err
function M.list_compressed_tar(args, skip, limit)
	local src, dst = M.spawn_7z_piped(
		{ "x", "-so", table.unpack(args) },
		{ "l", "-ba", "-slt", "-ttar", "-sccUTF-8", "-si" }
	)
	if not dst then
		return {}, 0, Err("Failed to start either `7zz` or `7z`. Do you have 7-zip installed?")
	end

	local files, bound, err = M.parse_7z_slt(dst, skip, limit)
	src:start_kill()
	dst:start_kill()

	return files, bound, err
end

function M.list_if_only_one(path)
	-- For certain compressed tarballs (e.g. .tar.xz),
	-- 7-zip doesn't print a .tar file if -slt is specified, so we are not doing that here
	local child = M.spawn_7z { "l", "-ba", "-sccUTF-8", "-p", tostring(path) }
	if not child then
		return false
	end

	local files = {}
	while #files < 2 do
		local next, event = child:read_line()
		if event == 0 then
			local attr, size, packed_size, path = next:sub(20):match("([^ ]+) +(%d+) +(%d+) +([^\r\n]+)")
			if path then
				files[#files + 1] = { path = path, size = tonumber(size), packed_size = tonumber(packed_size), attr = attr }
			end
		elseif event ~= 1 then
			break
		end
	end

	child:start_kill()
	if #files == 1 then
		return files[1]
	end
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

function M.should_decompress_tar(file)
	return file.packed_size <= 1024 * 1024 * 1024 and file.path:lower():find(".+%.tar$") ~= nil
end

-- Parse the output of a "7z l -slt" command.
-- The caller is responsible for killing the child process right after the execution of this function
---@param child Child
---@param skip integer
---@param limit integer
---@return table files
---@return integer bound
---@return Error? err
function M.parse_7z_slt(child, skip, limit)
	local i, files, err = 0, { { path = "", size = 0, packed_size = 0, attr = "" } }, nil
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
				files[#files + 1] = { path = "", size = 0, packed_size = 0, attr = "" }
			end
			goto continue
		elseif i < skip then
			goto continue
		end

		key, value = next:match("^(%u[%a ]+) = (.-)[\r\n]+")
		if key == "Path" then
			files[#files].path = value
		elseif key == "Size" then
			files[#files].size = tonumber(value) or 0
		elseif key == "Packed Size" then
			files[#files].packed_size = tonumber(value) or 0
		elseif key == "Attributes" then
			files[#files].attr = value
			if value:sub(1, 1) == "D" then
				files[#files].is_dir = true
			end
		elseif key == "Folder" and value == "+" then
			-- Mark as directory if Folder = +
			-- this is needed for some archive types, where Attributes may not have the D flag (ex: tarballs)
			files[#files].is_dir = true
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

function M.prepare_tree(files)
	if #files == 0 then
		return
	end

	local result = {}
	local dir_stack, previous_path = {}, nil

	-- Initialize dir_stack with the first file's
	local first_file_url, components = Url(files[1].path).path, {}
	while true do
		first_file_url = first_file_url.parent
		if first_file_url then
			components[#components + 1] = first_file_url
		else
			break
		end
	end
	for i, comp in ipairs(components) do
		table.insert(files, 1, {
			path = tostring(comp),
			size = 0,
			attr = "",
			is_dir = true,
			display_name = comp.name,
			depth = #components - i,
		})
	end

	local path, parent, dirs_to_add, dir_path = nil, nil, nil, nil
	for _, f in ipairs(files) do
		path = Url(f.path).path

		while #dir_stack > 0 and not path:starts_with(dir_stack[#dir_stack]) do
			dir_stack[#dir_stack] = nil
		end

		f.display_name = path.name
		f.depth = #dir_stack

		if f.is_dir then
			dir_stack[#dir_stack + 1] = path
		elseif not previous_path or (path.parent ~= previous_path.parent) then
			parent = path.parent
			dirs_to_add = {}
			while parent and (not dir_stack[#dir_stack] or parent ~= dir_stack[#dir_stack]) do
				dirs_to_add[#dirs_to_add + 1] = parent
				parent = parent.parent
			end
			for j = #dirs_to_add, 1, -1 do
				dir_path = dirs_to_add[j]
				result[#result + 1] = {
					path = tostring(dir_path),
					size = 0,
					attr = "",
					is_dir = true,
					display_name = dir_path.name,
					depth = #dir_stack,
				}
				dir_stack[#dir_stack + 1] = dir_path
			end
			f.depth = #dir_stack
		end

		result[#result + 1] = f
		previous_path = path
	end

	for i = #result, 1, -1 do
		files[i] = result[i]
	end
end

return M
