local M = {}

function M:peek()
	local child
	if ya.target_os() == "macos" then
		child = self:try_spawn("7zz") or self:try_spawn("7z")
	else
		child = self:try_spawn("7z") or self:try_spawn("7zz")
	end

	if not child then
		return ya.err("spawn `7z` and `7zz` both commands failed, error code: " .. tostring(self.last_error))
	end

	local limit = self.area.h
	local i, icon, names, sizes = 0, nil, {}, {}
	repeat
		local next, event = child:read_line()
		if event ~= 0 then
			break
		end

		local attr, size, name = next:match("^[-%d]+%s+[:%d]+%s+([.%a]+)%s+(%d+)%s+%d+%s+(.+)[\r\n]+")
		if not name then
			goto continue
		end

		i = i + 1
		if i <= self.skip then
			goto continue
		end

		icon = File({
			url = Url(name),
			cha = Cha { kind = attr:sub(1, 1) == "D" and 1 or 0 },
		}):icon()

		if icon then
			names[#names + 1] = ui.Line { ui.Span(" " .. icon.text .. " "):style(icon.style), ui.Span(name) }
		else
			names[#names + 1] = ui.Line(name)
		end

		size = tonumber(size)
		if size > 0 then
			sizes[#sizes + 1] = ui.Line(string.format(" %s ", ya.readable_size(size)))
		else
			sizes[#sizes + 1] = ui.Line("")
		end

		::continue::
	until i >= self.skip + limit

	child:start_kill()
	if self.skip > 0 and i < self.skip + limit then
		ya.manager_emit("peek", { math.max(0, i - limit), only_if = self.file.url, upper_bound = true })
	else
		ya.preview_widgets(self, {
			ui.Paragraph(self.area, names),
			ui.Paragraph(self.area, sizes):align(ui.Paragraph.RIGHT),
		})
	end
end

function M:seek(units)
	local h = cx.active.current.hovered
	if h and h.url == self.file.url then
		local step = math.floor(units * self.area.h / 10)
		ya.manager_emit("peek", {
			math.max(0, cx.active.preview.skip + step),
			only_if = self.file.url,
		})
	end
end

function M:try_spawn(name)
	local child, code = Command(name):args({ "l", "-ba", tostring(self.file.url) }):stdout(Command.PIPED):spawn()
	if not child then
		self.last_error = code
	end
	return child
end

return M
