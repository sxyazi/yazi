local M = {}

function M:cache() return ya.cache_file(self.file.url .. self.skip .. tostring(self.file.cha.modified)) end

function M:peek()
	if self:preload() == 1 then
		ya.image_show(self:cache(), self.area)
		ya.preview_widgets(self, {})
	end
end

function M:seek(units)
	local h = cx.active.current.hovered
	if h and h.url == self.file.url then
		ya.manager_emit("peek", {
			tostring(math.max(0, cx.active.preview.skip + units)),
			only_if = tostring(self.file.url),
		})
	end
end

function M:preload()
	local percentage = 5 + self.skip
	if percentage > 95 then
		ya.manager_emit("peek", { "90", only_if = tostring(self.file.url), upper_bound = "" })
		return 2
	end

	local cache = self:cache()
	if fs.symlink_metadata(cache) then
		return 1
	end

	local child = Command("ffmpegthumbnailer"):args({
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
		return 0
	end

	local status = child:wait()
	return status and status:success() and 1 or 2
end

return M
