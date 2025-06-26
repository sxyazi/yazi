local function fail(s, ...) error(string.format(s, ...)) end

local M = {}

function M:setup()
	ps.sub_remote("extract", function(args)
		local noisy = #args == 1 and ' "" --noisy' or ' ""'
		for _, arg in ipairs(args) do
			ya.emit("plugin", { self._id, ya.quote(arg, true) .. noisy })
		end
	end)
end

function M:entry(job)
	local from = job.args[1] and Url(job.args[1])
	local to = job.args[2] ~= "" and Url(job.args[2]) or nil
	if not from then
		fail("No URL provided")
	end

	local pwd = ""
	while true do
		if not M:try_with(from, pwd, to) then
			break
		elseif not job.args.noisy then
			fail("'%s' is password-protected, please extract it individually and enter the password", from)
		end

		local value, event = ya.input {
			pos = { "center", w = 50 },
			title = string.format('Password for "%s":', from.name),
			obscure = true,
		}
		if event == 1 then
			pwd = value
		else
			break
		end
	end
end

function M:try_with(from, pwd, to)
	to = to or from.parent
	if not to then
		fail("Invalid URL '%s'", from)
	end

	local tmp = fs.unique_name(to:join(self.tmp_name(from)))
	if not tmp then
		fail("Failed to determine a temporary directory for %s", from)
	end

	local archive = require("archive")
	local child, err = archive.spawn_7z { "x", "-aou", "-sccUTF-8", "-p" .. pwd, "-o" .. tostring(tmp), tostring(from) }
	if not child then
		fail("Failed to start both `7zz` and `7z`, error: " .. err)
	end

	local output, err = child:wait_with_output()
	if output and output.status.code == 2 and archive.is_encrypted(output.stderr) then
		fs.remove("dir_all", tmp)
		return true -- Need to retry
	end

	self:tidy(from, to, tmp)
	if not output then
		fail("7zip failed to output when extracting '%s', error: %s", from, err)
	elseif output.status.code ~= 0 then
		fail("7zip exited when extracting '%s', error code %s", from, output.status.code)
	end
end

function M:tidy(from, to, tmp)
	local outs = fs.read_dir(tmp, { limit = 2 })
	if not outs then
		fail("Failed to read the temporary directory '%s' when extracting '%s'", tmp, from)
	elseif #outs == 0 then
		fs.remove("dir", tmp)
		fail("No files extracted from '%s'", from)
	end

	local only = #outs == 1
	if only and not outs[1].cha.is_dir and require("archive").is_tar(outs[1].url) then
		self:entry { args = { tostring(outs[1].url), tostring(to) } }
		fs.remove("file", outs[1].url)
		fs.remove("dir", tmp)
		return
	end

	local target
	if only then
		target = to:join(outs[1].name)
	else
		target = to:join(self.trim_ext(from.name))
	end

	target = fs.unique_name(target)
	if not target then
		fail("Failed to determine a target for '%s'", from)
	end

	target = tostring(target)
	if only and not os.rename(tostring(outs[1].url), target) then
		fail('Failed to move "%s" to "%s"', outs[1].url, target)
	elseif not only and not os.rename(tostring(tmp), target) then
		fail('Failed to move "%s" to "%s"', tmp, target)
	end
	fs.remove("dir", tmp)
end

function M.tmp_name(url) return ".tmp_" .. ya.hash(string.format("extract//%s//%.10f", url, ya.time())) end

function M.trim_ext(name)
	-- stylua: ignore
	local exts = { ["7z"] = true, apk = true, bz2 = true, bzip2 = true, cbr = true, cbz = true, exe = true, gz = true, gzip = true, iso = true, jar = true, rar = true, tar = true, tgz = true, xz = true, zip = true, zst = true }

	while true do
		local s = name:gsub("%.([a-zA-Z0-9]+)$", function(s) return (exts[s] or exts[s:lower()]) and "" end)
		if s == name or s == "" then
			break
		else
			name = s
		end
	end
	return name
end

return M
