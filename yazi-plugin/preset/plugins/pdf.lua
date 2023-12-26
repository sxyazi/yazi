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
		local step = ya.clamp(-1, units, 1)
		ya.manager_emit("peek", { tostring(math.max(0, cx.active.preview.skip + step)), only_if = tostring(self.file.url) })
	end
end

function M:preload()
	local cache = self:cache()
	if fs.symlink_metadata(cache) then
		return 1
	end

	local output = Command.new("pdftoppm")
		:args({ "-singlefile", "-jpeg", "-jpegopt", "quality=75", "-f", tostring(self.skip + 1), tostring(self.file.url) })
		:stdout(Command.PIPED)
		:stderr(Command.PIPED)
		:output()

	if not output.status:success() then
		local pages = tonumber(output.stderr:match("the last page %((%d+)%)")) or 0
		if self.skip > 0 and pages > 0 then
			ya.manager_emit("peek", { tostring(math.max(0, pages - 1)), only_if = tostring(self.file.url), upper_bound = "" })
		end
		return 0
	end

	return fs.write(cache, output.stdout) and 1 or 2
end

return M
