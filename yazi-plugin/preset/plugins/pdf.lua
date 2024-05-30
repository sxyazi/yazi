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
		local step = ya.clamp(-1, units, 1)
		ya.manager_emit("peek", { math.max(0, cx.active.preview.skip + step), only_if = self.file.url })
	end
end

function M:preload()
	local cache = ya.file_cache(self)
	if not cache or fs.cha(cache) then
		return 1
	end

	local output = Command("pdftoppm")
		:args({ "-singlefile", "-jpeg", "-jpegopt", "quality=75", "-f", tostring(self.skip + 1), tostring(self.file.url) })
		:stdout(Command.PIPED)
		:stderr(Command.PIPED)
		:output()

	if not output then
		return 0
	elseif not output.status.success then
		local pages = tonumber(output.stderr:match("the last page %((%d+)%)")) or 0
		if self.skip > 0 and pages > 0 then
			ya.manager_emit("peek", { math.max(0, pages - 1), only_if = self.file.url, upper_bound = true })
		end
		return 0
	end

	return fs.write(cache, output.stdout) and 1 or 2
end

return M
