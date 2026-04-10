local function fail(s, ...) error(string.format(s, ...)) end

local M = {}

function M:setup()
	ps.sub_remote("extract", function(args)
		ya.async(function()
			for i, arg in ipairs(args) do
				local in_ = {
					self._id,
					args = { arg, "", noisy = #args == 1 },
					track = i == 1,
				}
				ya.task("plugin", in_):name("Extract " .. arg):spawn()
			end
		end)
	end)
end

function M:init(job)
	local from = job.args[1] and Url(job.args[1])
	if not from then
		fail("No URL provided")
	end

	local to = job.args[2] ~= "" and Url(job.args[2]) or from.parent
	if not to then
		fail("Failed to determine target directory for '%s'", from)
	end

	self.job = { id = job.id, from = from, to = to }
end

function M:entry(job)
	self:init(job)

	local pwd, target, retry = "", nil, false
	while true do
		target, retry = self:try_with(pwd)
		if not retry then
			break
		elseif not job.args.noisy then
			fail("'%s' is password-protected, please extract it individually and enter the password", self.job.from)
		end

		local value, event = ya.input {
			pos = { "top-center", y = 2, w = 50 },
			title = string.format('Password for "%s":', self.job.from.name),
			obscure = true,
		}
		if event == 1 then
			pwd = value
		else
			break
		end
	end

	if target then
		ya.emit("tasks:update_succeed", { job.id, urls = { target }, track = true })
	end
end

function M:try_with(pwd)
	local from, to = self.job.from, self.job.to
	local tmp = fs.unique("dir", to:join(self.tmp_name(from)))
	if not tmp then
		fail("Failed to determine a temporary directory for %s", from)
	end

	local archive = require("archive")
	local child, err = archive.spawn_7z { "x", "-aou", "-sccUTF-8", "-p" .. pwd, "-o" .. tostring(tmp), tostring(from) }
	if not child then
		fail("Failed to start either `7zz` or `7z`, error: " .. err)
	end

	local output, err = child:wait_with_output()
	if output and output.status.code == 2 and archive.is_encrypted(output.stderr) then
		fs.remove("dir_all", tmp)
		return nil, true -- Need to retry
	end

	local target = self:tidy(tmp)
	if not output then
		fail("7zip failed to output when extracting '%s', error: %s", from, err)
	elseif output.status.code ~= 0 then
		fail("7zip exited with error code %s when extracting '%s':\n%s", output.status.code, from, output.stderr)
	else
		return target, false
	end
end

function M:tidy(tmp)
	local from, to = self.job.from, self.job.to
	local outs = fs.read_dir(tmp, { limit = 2 })
	if not outs then
		fail("Failed to read the temporary directory '%s' when extracting '%s'", tmp, from)
	elseif #outs == 0 then
		fs.remove("dir", tmp)
		fail("No files extracted from '%s'", from)
	end

	local only = #outs == 1 and outs[1]
	if only and not only.cha.is_dir and require("archive").is_tar(only.url) then
		self:entry { id = self.job.id, args = { tostring(only.url), tostring(to) } }
		fs.remove("file", only.url)
		fs.remove("dir", tmp)
		return
	end

	local target = to:join(only and only.name or self.trim_ext(from.name))
	target = fs.unique(only and not only.cha.is_dir and "file" or "dir", target)
	if not target then
		fail("Failed to determine a target for '%s'", from)
	end

	if only and not fs.rename(only.url, target) then
		fail('Failed to move "%s" to "%s"', only.url, target)
	elseif not only and not fs.rename(tmp, target) then
		fail('Failed to move "%s" to "%s"', tmp, target)
	end

	fs.remove("dir", tmp)
	return target
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
