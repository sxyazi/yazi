local M = {}

function M:peek()
	local start, url = os.clock(), ya.file_cache(self)
	if not url or not fs.cha(url) then
		url = self.file.url
	end

	ya.sleep(math.max(0, PREVIEW.image_delay / 1000 + start - os.clock()))
	ya.image_show(url, self.area)
	ya.preview_widgets(self, {})
end

function M:seek() end

function M:preload()
	local cache = ya.file_cache(self)
	if not cache or fs.cha(cache) then
		return 1
	end

	return ya.image_precache(self.file.url, cache) and 1 or 2
end

return M
