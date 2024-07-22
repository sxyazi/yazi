local SUPPORTED_TYPES = "application/audio/biosig/chemical/font/image/inode/message/model/rinex/text/vector/video/x-epoc/"

local M = {}

local function match_mimetype(s)
	local type, sub = s:match("([-a-z]+/)([+-.a-zA-Z0-9]+)%s*$")
	if type and sub and SUPPORTED_TYPES:find(type, 1, true) then
		return type .. sub
	end
end

function M:fetch()
	local urls = {}
	for _, file in ipairs(self.files) do
		urls[#urls + 1] = tostring(file.url)
	end

	local cmd = os.getenv("YAZI_FILE_ONE") or "file"
	local child, code = Command(cmd):args({ "-bL", "--mime-type" }):args(urls):stdout(Command.PIPED):spawn()
	if not child then
		ya.err(string.format("Spawn `%s` command returns %s", cmd, code))
		return 0
	end

	local updates, last = {}, ya.time()
	local flush = function(force)
		if not force and ya.time() - last < 0.3 then
			return
		end
		if next(updates) then
			ya.manager_emit("update_mimetype", { updates = updates })
			updates, last = {}, ya.time()
		end
	end

	local i, j, valid = 1, 0, nil
	repeat
		local line, event = child:read_line_with { timeout = 300 }
		if event == 3 then
			flush(true)
			goto continue
		elseif event ~= 0 then
			break
		end

		valid = match_mimetype(line)
		if valid and line:find(valid, 1, true) ~= 1 then
			goto continue
		elseif valid then
			j, updates[urls[i]] = j + 1, valid
			flush(false)
		end

		i = i + 1
		::continue::
	until i > #urls

	flush(true)
	return j == #urls and 3 or 2
end

-- TODO: remove this after v0.3 release
local notified = ya.sync(function (state)
	if state.notified then
		return true
	else
		state.notified = true
		return false
	end
end)
function M:preload()
	if notified() then
		return 1
	end
	ya.notify {
		title = "Error",
		content = [[In Yazi v0.3, the `mime` plugin has been re-classified as a fetcher. Please remove it from the `preloaders` of your yazi.toml

See https://github.com/sxyazi/yazi/issues/1046 for details.]],
		timeout = 20,
		level = "error",
	}
	return 1
end

return M
