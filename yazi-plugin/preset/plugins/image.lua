local M = {}

function M:cache() return ya.cache_file(self.file.url .. tostring(self.file.cha.modified)) end

function M:peek()
	local cache = self:cache()
	ya.image_show(fs.symlink_metadata(cache) and cache or self.file.url, self.area)
	ya.preview_widgets(self, {})
end

function M:seek() end

function M:preload()
	local cache = self:cache()
	if fs.symlink_metadata(cache) then
		return 1
	end

	return ya.image_precache(self.file.url, cache) and 1 or 2
end

return M
