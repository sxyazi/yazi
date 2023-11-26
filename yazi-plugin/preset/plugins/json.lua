local Json = {}

function Json:peek()
	local limit = self.area.h
	local child = Command.new("jq")
		:args({
			"-C",
			"--tab",
			".",
			tostring(self.file.url),
		})
		:stdout(Command.PIPED)
		:stderr(Command.PIPED)
		:spawn()

	local i, lines = 0, ""
	repeat
		local code, next = child:read_line()
		if code ~= 0 then
			break
		end

		i = i + 1
		if i > self.skip then
			lines = lines .. next
		end
	until i >= self.skip + limit

	child:start_kill()
	if self.skip > 0 and i < self.skip + limit then
		ya.manager_emit("peek", { tostring(math.max(0, i - limit)) })
	else
		lines = lines:gsub("\t", string.rep(" ", PREVIEW.tab_size))
		ya.preview_widgets(self.file, self.skip, { ui.Paragraph.parse(self.area, lines) })
	end
end

function Json:seek(units)
	local h = cx.active.current.hovered
	if h and h.url == self.file.url then
		local step = math.floor(units * self.area.h / 10)
		ya.manager_emit("peek", { tostring(math.max(0, cx.active.preview.skip + step)) })
	end
end

return Json
