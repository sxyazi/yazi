local M = {}

function M:peek()
	local url = ya.file_cache(self)
	if not url or not fs.cha(url) then
		url = self.file.url
	end

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
