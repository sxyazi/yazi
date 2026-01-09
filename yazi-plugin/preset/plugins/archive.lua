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
		files = { { path = Path.os(job.file.url.stem), size = 0, packed_size = 0, attr = "" } }
	end

	files = M.prepare_tree(files)

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
				files[#files + 1] =
					{ path = Path.os(path), size = tonumber(size), packed_size = tonumber(packed_size), attr = attr }
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
	return file.packed_size <= 1024 * 1024 * 1024 and (file.path.ext or ""):lower() == "tar"
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
	local i, files, err = 0, { { path = Path.os(""), size = 0, packed_size = 0, attr = "" } }, nil
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
			if files[#files].path ~= Path.os("") then
				files[#files + 1] = { path = Path.os(""), size = 0, packed_size = 0, attr = "" }
			end
			goto continue
		elseif i < skip then
			goto continue
		end

		key, value = next:match("^(%u[%a ]+) = (.-)[\r\n]+")
		if key == "Path" then
			files[#files].path = Path.os(value)
		elseif key == "Size" then
			files[#files].size = tonumber(value) or 0
		elseif key == "Packed Size" then
			files[#files].packed_size = tonumber(value) or 0
		elseif key == "Attributes" then
			files[#files].attr = value
		elseif key == "Folder" then
			files[#files].folder = value
		end

		::continue::
	until i >= skip + limit

	if files[#files].path == Path.os("") then
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

	local parent = files[1].path.parent
	while parent do
		table.insert(files, 1, {
			path = parent,
			size = 0,
			attr = "",
			is_dir = true,
			display_name = parent.name,
		})
		parent = parent.parent
	end

	local tree = {}
	local parents, prev_parent = {}, nil
	for i, f in ipairs(files) do
		if f.is_dir then
			f.depth = i - 1
		else
			f.is_dir = f.folder == "+" or (f.attr and f.attr:sub(1, 1) == "D")
		end

		while #parents > 0 and not f.path:starts_with(parents[#parents]) do
			parents[#parents] = nil
		end

		f.display_name = f.path.name
		f.depth = #parents

		if f.is_dir then
			parents[#parents + 1] = f.path
		elseif prev_parent ~= f.path.parent then
			local parent = f.path.parent
			local dirs_to_add = {}
			while parent and parent ~= parents[#parents] do
				dirs_to_add[#dirs_to_add + 1] = parent
				parent = parent.parent
			end
			for j = #dirs_to_add, 1, -1 do
				tree[#tree + 1] = {
					path = dirs_to_add[j],
					size = 0,
					attr = "",
					is_dir = true,
					display_name = dirs_to_add[j].name,
					depth = #parents,
				}
				parents[#parents + 1] = dirs_to_add[j]
			end
			f.depth = #parents
		end

		tree[#tree + 1] = f
		prev_parent = f.path.parent
	end

	return tree
end

return M
