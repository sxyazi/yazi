local M = {}

function M:peek()
	local cache = ya.file_cache(self)
	if not cache then
		return
	end

	if self:preload() == 1 then
		ya.image_show(cache, self.area)
		ya.preview_widgets(self, {})
	end
end

function M:seek(units)
	local h = cx.active.current.hovered
	if h and h.url == self.file.url then
		ya.manager_emit("peek", {
			math.max(0, cx.active.preview.skip + units),
			only_if = self.file.url,
		})
	end
end

function M:preload()
	local percentage = 5 + self.skip
	if percentage > 95 then
		ya.manager_emit("peek", { 90, only_if = self.file.url, upper_bound = true })
		return 2
	end

	local cache = ya.file_cache(self)
	if not cache or fs.cha(cache) then
		return 1
	end

	local child, code = Command("ffmpegthumbnailer"):args({
		"-q",
		"6",
		"-c",
		"jpeg",
		"-i",
		tostring(self.file.url),
		"-o",
		tostring(cache),
		"-t",
		tostring(percentage),
		"-s",
		tostring(PREVIEW.max_width),
	}):spawn()

	if not child then
		ya.err("spawn `ffmpegthumbnailer` command returns " .. tostring(code))
		return 0
	end

	local status = child:wait()
	return status and status.success and 1 or 2
end

return M
