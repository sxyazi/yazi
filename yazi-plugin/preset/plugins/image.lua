local Image = {}

function Image:cache() return ya.cache_file(self.file.url .. tostring(self.file.cha.modified)) end

function Image:peek()
	local cache = self:cache()
	ya.image_show(fs.symlink_metadata(cache) and cache or self.file.url, self.area)
	ya.preview_widgets(self.file, self.skip, {})
end

function Image:seek() end

function Image:preload()
	local cache = self:cache()
	if fs.symlink_metadata(cache) then
		return 1
	end

	return ya.image_precache(self.file.url, cache) and 1 or 2
end

return Image
