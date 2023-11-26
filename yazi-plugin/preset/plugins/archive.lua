local Archive = {}

function Archive:peek()
	local _, max = ya.preview_archive(self.area, self.file, self.skip)
	if max then
		ya.manager_emit("peek", { tostring(max) })
	end
end

function Archive:seek(units)
	local h = cx.active.current.hovered
	if h and h.url == self.file.url then
		local step = math.floor(units * self.area.h / 10)
		ya.manager_emit("peek", { tostring(math.max(0, cx.active.preview.skip + step)) })
	end
end

return Archive
