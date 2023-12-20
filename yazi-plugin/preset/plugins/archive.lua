local Archive = {}

function Archive:peek()
	local _, bound = ya.preview_archive(self)
	if bound then
		ya.manager_emit("peek", { tostring(bound), only_if = tostring(self.file.url), upper_bound = "" })
	end
end

function Archive:seek(units)
	local h = cx.active.current.hovered
	if h and h.url == self.file.url then
		local step = math.floor(units * self.area.h / 10)
		ya.manager_emit("peek", {
			tostring(math.max(0, cx.active.preview.skip + step)),
			only_if = tostring(self.file.url),
		})
	end
end

return Archive
