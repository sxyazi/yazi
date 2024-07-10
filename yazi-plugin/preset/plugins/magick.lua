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

function M:seek() end

function M:preload()
	local cache = ya.file_cache(self)
	if not cache or fs.cha(cache) then
		return 1
	end

	local child, code = Command("magick"):args({
		"-density",
		"200",
		tostring(self.file.url),
		"-resize",
		string.format("%dx%d^", PREVIEW.max_width, PREVIEW.max_height),
		"-quality",
		tostring(PREVIEW.image_quality),
		"JPG:" .. tostring(cache),
	}):spawn()

	if not child then
		ya.err("spawn `magick` command returns " .. tostring(code))
		return 0
	end

	local status = child:wait()
	return status and status.success and 1 or 2
end

return M
