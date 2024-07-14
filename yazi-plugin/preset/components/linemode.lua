Linemode = {
	_inc = 1000,
}

function Linemode:solo(file)
	local mode = cx.active.conf.linemode
	if mode == "none" or mode == "solo" then
		return ui.Line("")
	end

	if not self[mode] then
		return ui.Line(" " .. mode .. " ")
	end

	return ui.Line {
		ui.Span(" "),
		self[mode](self, file),
		ui.Span(" "),
	}
end

function Linemode:size(file)
	local size = file:size()
	return ui.Line(size and ya.readable_size(size) or "")
end

function Linemode:ctime(file)
	local time = (file.cha.created or 0) // 1
	if time == 0 then
		return ui.Line("")
	elseif os.date("%Y", time) == os.date("%Y") then
		return ui.Line(os.date("%m/%d %H:%M", time))
	else
		return ui.Line(os.date("%m/%d  %Y", time))
	end
end

function Linemode:mtime(file)
	local time = (file.cha.modified or 0) // 1
	if time == 0 then
		return ui.Line("")
	elseif os.date("%Y", time) == os.date("%Y") then
		return ui.Line(os.date("%m/%d %H:%M", time))
	else
		return ui.Line(os.date("%m/%d  %Y", time))
	end
end

function Linemode:permissions(file) return ui.Line(file.cha:permissions() or "") end

function Linemode:owner(file)
	local user = file.cha.uid and ya.user_name(file.cha.uid) or file.cha.uid
	local group = file.cha.gid and ya.group_name(file.cha.gid) or file.cha.gid
	return ui.Line(string.format("%s:%s", user or "-", group or "-"))
end

function Linemode:render(files)
	local lines = {}
	for _, f in ipairs(files) do
		lines[#lines + 1] = self:children_render(f)
	end
	return lines
end

-- Initialize children
Linemode._children = {
	{ Linemode.solo, id = 1, order = 1000 },
}

function Linemode:children_add(fn, order)
	self._inc = self._inc + 1
	self._children[#self._children + 1] = { fn, id = self._inc, order = order }
	table.sort(self._children, function(a, b) return a.order < b.order end)
	return self._inc
end

function Linemode:children_remove(id)
	for i, child in ipairs(self._children) do
		if child.id == id then
			table.remove(self._children, i)
			break
		end
	end
end

function Linemode:children_render(file)
	local lines = {}
	for _, child in ipairs(self._children) do
		lines[#lines + 1] = child[1](self, file)
	end
	return ui.Line(lines)
end
