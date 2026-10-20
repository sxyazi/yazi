local function fail(s, ...) error(string.format(s, ...)) end

local M = {}

function M:new(from, to, noisy)
	if not from then
		fail("No URL provided")
	elseif not to then
		fail("Failed to determine target directory for '%s'", from)
	end

	return setmetatable({
		from = from,
		to = to,
		noisy = noisy,
		shared = self.shared or { tmps = {} },
	}, { __index = M })
end

function M:setup()
	ps.sub_remote("extract", function(args)
		for i, arg in ipairs(args) do
			ya.async(function()
				local from = Url(arg)

				local me = self:new(from, from.parent, #args == 1)
				local task = me:task(i == 1)

				local ok, dest = pcall(me.run, me)
				local cleaned, err = me:cleanup()
				if not cleaned then
					task:progress { failed = 1 }
					task:fail(tostring(err))
				elseif not ok then
					task:progress { failed = 1 }
					task:fail(tostring(dest))
				elseif dest then
					task:progress { succeeded = 1 }
					task:succeed { dest }
				else
					task:cancel()
				end
			end)
		end
	end)
end

function M:task(track)
	local in_ = {
		pool = "plugin",
		scope = rt.scope():child(),
		track = track,
		progress = true,
	}

	self.shared.task = ya.task("custom", in_):name("Extract " .. tostring(self.from)):spawn()
	self.shared.task:progress { total = 1 }
	return self.shared.task
end

function M:run()
	if not self.shared.task:acquire() then
		return
	end

	local branch, tmp = ya.select(self.shared.task, function() return self:unpack() end)
	if branch == 2 and tmp then
		return self:place(tmp)
	end
end

function M:unpack()
	local pwd = ""
	while true do
		local tmp, retry = self:try_unpack(pwd)
		if not retry then
			return tmp
		elseif not self.noisy then
			fail("'%s' is password-protected, please extract it individually and enter the password", self.from)
		end

		local value, event = ya.input {
			pos = { "top-center", y = 2, w = 50 },
			title = string.format('Password for "%s":', self.from.name),
			obscure = true,
		}
		if event == 1 then
			pwd = value
		else
			return
		end
	end
end

function M:try_unpack(pwd)
	local tmp = fs.unique("dir", self.to:join(self.tmp_name(self.from)))
	if not tmp then
		fail("Failed to determine a temporary directory for %s", self.from)
	end
	self.shared.tmps[#self.shared.tmps + 1] = tmp

	local archive = require("archive")
	local child, err = archive.spawn_7z {
		"x",
		"-aou",
		"-sccUTF-8",
		"-bso0",
		"-bsp1",
		"-p" .. pwd,
		"-o" .. tostring(tmp),
		tostring(self.from),
	}
	if not child then
		fail("Failed to start either `7zz` or `7z`, error: " .. err)
	end
	self.shared.child = child
	self.shared.task:progress { workload = 100 }

	local chunks, tail, cur, last = {}, "", 0, 0
	while true do
		local chunk, event, err = child:read(4096)
		if not chunk then
			fail("7zip failed to output when extracting '%s', error: %s", self.from, err)
		elseif event == 2 then
			break
		elseif event == 1 then
			chunks[#chunks + 1] = chunk
		else
			cur, tail = self.parse_progress(tail .. chunk)
			if cur > last then
				self.shared.task:progress { processed = cur - last }
				last = cur
			end
		end
	end

	local status, err = child:wait()
	if not status then
		fail("7zip failed to output when extracting '%s', error: %s", self.from, err)
	end
	self.shared.task:progress { processed = 100 - last }

	local stderr = table.concat(chunks)
	if status.code == 2 and archive.is_encrypted(stderr) then
		return nil, true -- Needs a retry
	elseif status.code ~= 0 then
		fail("7zip exited with error code %s when extracting '%s':\n%s", status.code, self.from, stderr)
	end

	return tmp
end

function M:place(tmp)
	local outs = fs.read_dir(tmp, { limit = 2 })
	if not outs then
		fail("Failed to read the temporary directory '%s' when extracting '%s'", tmp, self.from)
	elseif #outs == 0 then
		fail("No files extracted from '%s'", self.from)
	end

	local only = #outs == 1 and outs[1]
	if only and not only.stat.is_dir and require("archive").is_tar(only.url) then
		return self:new(only.url, self.to):run()
	end

	local kind = only and not only.stat.is_dir and "file" or "dir"
	local dest = fs.unique(kind, self.to:join(only and only.name or self.trim_ext(self.from.name)))
	if not dest then
		fail("Failed to determine a target for '%s'", self.from)
	end

	local src = only and only.url or tmp
	if not fs.rename(src, dest) then
		fs.remove(kind, dest)
		fail('Failed to move "%s" to "%s"', src, dest)
	end

	return dest
end

function M:cleanup()
	local child = self.shared.child
	if child then
		child:start_kill()
		local status, err = child:wait()
		if not status then
			return false, err
		end
	end

	for _, tmp in ipairs(self.shared.tmps) do
		fs.remove("dir_all", tmp)
	end
	return true
end

function M.tmp_name(url) return ".tmp_" .. ya.hash(string.format("extract//%s//%.10f", url, ya.time().unix)) end

function M.trim_ext(name)
	-- stylua: ignore
	local exts = { ["7z"] = true, apk = true, bz2 = true, bzip2 = true, cbr = true, cbz = true, exe = true, gz = true, gzip = true, iso = true, jar = true, rar = true, tar = true, tgz = true, xz = true, zip = true, zst = true }

	while true do
		local s = name:gsub("%.([a-zA-Z0-9]+)$", function(s) return exts[s:lower()] and "" end)
		if s == name or s == "" then
			return name
		end
		name = s
	end
end

function M.parse_progress(chunk)
	local percent = 0
	for s in chunk:gmatch("[^\r\n\b]+") do
		percent = tonumber(s:match("^%s*(%d+)%%")) or percent
	end
	return math.min(100, percent), chunk:match("[^\r\n\b]*$")
end

return M
