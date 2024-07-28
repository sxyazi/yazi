local function fail(s, ...) error(string.format(s, ...)) end

local M = {}

function M:setup()
	ps.sub_remote("extract", function(args)
		local noisy = #args == 1 and " --noisy" or ""
		for _, arg in ipairs(args) do
			ya.manager_emit("plugin", { self._id, args = ya.quote(arg, true) .. noisy })
		end
	end)
end

function M:entry(args)
	if not args[1] then
		fail("No URL provided")
	end

	local url, pwd = Url(args[1]), ""
	while true do
		if not M:try_with(url, pwd) then
			break
		elseif args[2] ~= "--noisy" then
			fail("'%s' is password-protected, please extract it individually and enter the password", args[1])
		end

		local value, event = ya.input {
			title = string.format('Password for "%s":', url:name()),
			position = { "center", w = 50 },
		}
		if event == 1 then
			pwd = value
		else
			break
		end
	end
end

function M:try_with(url, pwd)
	local parent = url:parent()
	if not parent then
		fail("Invalid URL '%s'", url)
	end

	local tmp = fs.unique_name(parent:join(self.tmp_name(url)))
	if not tmp then
		fail("Failed to determine a temporary directory for %s", url)
	end

	local archive = require("archive")
	local child, code = archive:spawn_7z { "x", "-aou", "-p" .. pwd, "-o" .. tostring(tmp), tostring(url) }
	if not child then
		fail("Spawn `7z` and `7zz` both commands failed, error code %s", code)
	end

	local output, err = child:wait_with_output()
	if output and output.status.code == 2 and archive:is_encrypted(output.stderr) then
		fs.remove("dir_clean", tmp)
		return true -- Needs retry
	end

	self:tidy(url, tmp)
	if not output then
		fail("7zip failed to output when extracting '%s', error code %s", err, url)
	elseif output.status.code ~= 0 then
		fail("7zip exited when extracting '%s', error code %s", url, output.status.code)
	end
end

function M:tidy(url, tmp)
	local files = fs.read_dir(tmp, { limit = 2 })
	if not files then
		fail("Failed to read the temporary directory '%s' when extracting '%s'", tmp, url)
	elseif #files == 0 then
		fs.remove("dir", tmp)
		fail("No files extracted from '%s'", url)
	end

	local target
	local only_dir = #files == 1 and files[1].cha.is_dir
	if only_dir then
		target = url:parent():join(files[1].name)
	else
		target = url:parent():join(self.trim_ext(url:name()))
	end

	target = fs.unique_name(target)
	if not target then
		fail("Failed to determine a target directory for '%s'", url)
	end

	if only_dir and not os.rename(tostring(files[1].url), tostring(target)) then
		fail('Failed to move "%s" to "%s"', files[1].url, target)
	elseif not only_dir and not os.rename(tostring(tmp), tostring(target)) then
		fail('Failed to move "%s" to "%s"', tmp, target)
	end
	fs.remove("dir", tmp)
end

function M.tmp_name(url) return ".tmp_" .. ya.md5(string.format("extract//%s//%.10f", url, ya.time())) end

function M.trim_ext(name)
	-- stylua: ignore
	local exts = { ["7z"] = true, apk = true, bz2 = true, bzip2 = true, exe = true, gz = true, gzip = true, iso = true, jar = true, rar = true, tar = true, tgz = true, xz = true, zip = true, zst = true }

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
