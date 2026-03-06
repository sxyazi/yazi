local M = {}

function M:peek(job)
	local limit = job.area.h
	local items, err = self.list_archive({ "-p", tostring(job.file.path) }, job.skip, limit)

	local first = (#items == 1 and items[1]) or (#items == 0 and M.list_if_only_one(job.file.path))
	if first and M.should_decompress_tar(job.file, first) then
		items, err = self.list_compressed_tar({ "-p", tostring(job.file.path) }, job.skip, limit)
	end

	if err then
		return ya.preview_widget(job, err)
	elseif job.skip > 0 and #items < job.skip + limit then
		return ya.emit("peek", { math.max(0, #items - limit), only_if = job.file.url, upper_bound = true })
	elseif #items == 0 then
		items = { M.make_item { path = job.file.url.stem } }
	end

	local left, right = {}, {}
	for i = job.skip + 1, #items do
		local f = items[i]
		local icon = File({
			url = Url(f.path),
			cha = Cha { mode = tonumber(f.is_dir and "40700" or "100644", 8) },
		}):icon()

		if f.size > 0 then
			right[#right + 1] = " " .. ya.readable_size(f.size) .. " "
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
			ui.truncate(f.path.name or tostring(f.path), {
				rtl = true,
				max = math.max(0, job.area.w - (f.depth * 2) - ui.width(left[#left]) - ui.width(right[#right])),
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

---List items in an archive
---@param args table
---@param skip integer
---@param limit integer
---@return table items
---@return Error? err
function M.list_archive(args, skip, limit)
	local child = M.spawn_7z { "l", "-ba", "-slt", "-sccUTF-8", "-xr!__MACOSX", table.unpack(args) }
	if not child then
		return {}, Err("Failed to start either `7zz` or `7z`. Do you have 7-zip installed?")
	end

	local items, err = M.parse_7z_slt(child, skip, limit)
	child:start_kill()

	return items, err
end

---List items in a compressed tarball
---@param args table
---@param skip integer
---@param limit integer
---@return table items
---@return Error? err
function M.list_compressed_tar(args, skip, limit)
	local src, dst = M.spawn_7z_piped(
		{ "x", "-so", table.unpack(args) },
		{ "l", "-ba", "-slt", "-ttar", "-sccUTF-8", "-xr!__MACOSX", "-si" }
	)
	if not dst then
		return {}, Err("Failed to start either `7zz` or `7z`. Do you have 7-zip installed?")
	end

	local items, err = M.parse_7z_slt(dst, skip, limit)
	src:start_kill()
	dst:start_kill()

	return items, err
end

---@param path Path
---@return table?
function M.list_if_only_one(path)
	-- For certain compressed tarballs (e.g. .tar.xz),
	-- 7-zip doesn't print a .tar item if -slt is specified, so we are not doing that here
	local child = M.spawn_7z { "l", "-ba", "-sccUTF-8", "-p", tostring(path) }
	if not child then
		return
	end

	local items = {}
	while #items < 2 do
		local next, event = child:read_line()
		if event == 0 then
			local attr = next:sub(21, 25)
			local size = next:sub(27, 38):gsub("^%s+", "")
			local packed_size = next:sub(40, 51):gsub("^%s+", "")
			local path = next:sub(54):gsub("\r?\n$", "")
			if path ~= "" then
				items[#items + 1] = M.make_item { path = path, size = size, packed_size = packed_size, attr = attr }
			end
		elseif event ~= 1 then
			break
		end
	end

	child:start_kill()
	if #items == 1 then
		return items[1]
	end
end

---List metadata of an archive
---@param args table
---@return string? type
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

function M.make_item(t)
	t = t or {}
	t.path = type(t.path or "") == "string" and Path.os(t.path or "") or t.path
	t.size = tonumber(t.size) or 0
	t.packed_size = tonumber(t.packed_size) or 0
	t.attr = t.attr or ""
	t.folder = t.folder or ""
	t.depth = tonumber(t.depth) or 0
	return t
end

---@param file File
---@param item table
---@return boolean
function M.should_decompress_tar(file, item)
	if (item.path.ext or ""):lower() ~= "tar" then
		return false
	elseif item.packed_size > 0 then
		return item.packed_size <= 1024 * 1024 * 1024
	else
		return file.cha.len <= 100 * 1024 * 1024
	end
end

-- Parse the output of a "7z l -slt" command.
-- The caller is responsible for killing the child process right after the execution of this function
---@param child Child
---@param skip integer
---@param limit integer
---@return table items
---@return Error? err
function M.parse_7z_slt(child, skip, limit)
	local items, tops, parents, err = { M.make_item() }, {}, {}, nil
	local key, value, empty, stderr = "", "", Path.os(""), {}
	repeat
		local next, event = child:read_line()
		if event == 1 and M.is_encrypted(next) then
			err = Err("File list of the archive is encrypted")
			break
		elseif event == 1 then
			stderr[#stderr + 1] = next
			goto continue
		elseif event ~= 0 then
			break
		end

		if next == "\n" or next == "\r\n" then
			if items[#items].path ~= empty then
				M.treelize(items, tops, parents)
				M.pop_dup_dir(items, parents, false)
				items[#items + 1] = M.make_item()
			end
			goto continue
		end

		key, value = next:match("^(%u[%a ]+) = (.-)[\r\n]+")
		if key == "Path" then
			items[#items].path = Path.os(value)
		elseif key == "Size" then
			items[#items].size = tonumber(value) or 0
		elseif key == "Packed Size" then
			items[#items].packed_size = tonumber(value) or 0
		elseif key == "Attributes" then
			items[#items].attr = value
		elseif key == "Folder" then
			items[#items].folder = value
		end

		::continue::
	until #items - 1 > skip + limit

	if items[#items].path == empty then
		items[#items] = nil
	else
		M.treelize(items, tops, parents)
	end

	M.pop_dup_dir(items, parents, #items <= skip + limit)
	if #items > skip + limit then
		items[#items] = nil
	end

	if #stderr ~= 0 then
		err = Err("7-zip errored out while listing items, stderr: %s", table.concat(stderr, "\n"))
	end
	return items, err
end

---Convert a flat list of items into a tree structure
---@param items table
---@param tops Path[]
---@param parents table<string, boolean>
function M.treelize(items, tops, parents)
	local f = table.remove(items)
	while #tops > 0 and not f.path:starts_with(tops[#tops]) do
		tops[#tops] = nil
	end

	local buf, it = {}, f.path.parent
	while it and it ~= tops[#tops] do
		buf[#buf + 1], it = it, it.parent
	end
	for i = #buf, 1, -1 do
		items[#items + 1] = M.make_item { path = buf[i], depth = #tops, is_dir = true }
		tops[#tops + 1] = buf[i]
		M.pop_dup_dir(items, parents, false)
	end

	f.depth = #tops
	f.is_dir = f.folder == "+" or f.attr:sub(1, 1) == "D"

	if not f.is_dir then
		items[#items + 1] = f
	elseif f.path ~= tops[#tops] then
		items[#items + 1], tops[#tops + 1] = f, f.path
	end
end

---@param items table
---@param parents table<string, boolean>
---@param eof boolean
function M.pop_dup_dir(items, parents, eof)
	local n, i = #items, eof and #items or #items - 1
	if not items[i] or not items[i].is_dir then
		return
	end

	local p = tostring(items[i].path)
	if not parents[p] then
		parents[p] = true
	elseif eof then
		items[n] = nil
	elseif not items[n].path:starts_with(items[i].path) then
		items[i], items[n] = items[n], nil
	end
end

return M
