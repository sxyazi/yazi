local M = {}

function M:peek()
	local start, cache = os.clock(), ya.file_cache(self)
	if not cache or self:preload() ~= 1 then
		return
	end

	ya.sleep(math.max(0, PREVIEW.image_delay / 1000 + start - os.clock()))
	ya.image_show(cache, self.area)
	ya.preview_widgets(self, {})
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
		"-auto-orient",
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
