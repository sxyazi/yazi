local M = {}

function M:peek(job)
	local limit = job.area.h

	local files, bound, code = self.list_files({ "-p", tostring(job.file.url) }, job.skip, limit)
	if code ~= 0 then
		return require("empty").msg(
			job,
			code == 2 and "File list in this archive is encrypted"
				or "Failed to start both `7zz` and `7z`. Do you have 7-zip installed?"
		)
	end

	local left, right = {}, {}
	for _, f in ipairs(files) do
		local icon = File({
			url = Url(f.path),
			cha = Cha { kind = f.attr:sub(1, 1) == "D" and 1 or 0 },
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
			ya.truncate(f.path, {
				rtl = true,
				max = math.max(0, job.area.w - ui.width(left[#left]) - ui.width(right[#right])),
			}),
		}
	end

	if job.skip > 0 and bound < job.skip + limit then
		ya.emit("peek", { math.max(0, bound - limit), only_if = job.file.url, upper_bound = true })
	else
		ya.preview_widget(job, {
			ui.Text(left):area(job.area),
			ui.Text(right):area(job.area):align(ui.Align.RIGHT),
		})
	end
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
		return ya.err("Failed to start both `7zz` and `7z`, error: " .. last_err)
	end
	return child, last_err
end

---List files in an archive
---@param args table
---@param skip integer
---@param limit integer
---@return table files
---@return integer bound
---@return integer code
---  0: success
---  1: failed to spawn
---  2: wrong password
---  3: partial success
function M.list_files(args, skip, limit)
	local child = M.spawn_7z { "l", "-ba", "-slt", "-sccUTF-8", table.unpack(args) }
	if not child then
		return {}, 0, 1
	end

	local i, files, code = 0, { { path = "", size = 0, attr = "" } }, 0
	local key, value = "", ""
	repeat
		local next, event = child:read_line()
		if event == 1 and M.is_encrypted(next) then
			code = 2
			break
		elseif event == 1 then
			code = 3
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
	child:start_kill()

	if files[#files].path == "" then
		files[#files] = nil
	end
	return files, i, code
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

function M.is_tar(url) return M.list_meta { "-p", tostring(url) } == "tar" end

return M
