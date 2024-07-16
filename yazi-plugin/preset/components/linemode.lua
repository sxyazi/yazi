Linemode = {
	_inc = 1000,
	_children = {
		{ "solo", id = 1, order = 1000 },
	},
}

function Linemode:new(file) return setmetatable({ _file = file }, { __index = self }) end

function Linemode:solo()
	local mode = cx.active.conf.linemode
	if mode == "none" or mode == "solo" then
		return ui.Line("")
	end

	if not self[mode] then
		return ui.Line(" " .. mode .. " ")
	end

	return ui.Line {
		ui.Span(" "),
		self[mode](self),
		ui.Span(" "),
	}
end

function Linemode:size()
	local size = self._file:size()
	return ui.Line(size and ya.readable_size(size) or "")
end

function Linemode:ctime()
	local time = (self._file.cha.created or 0) // 1
	if time == 0 then
		return ui.Line("")
	elseif os.date("%Y", time) == os.date("%Y") then
		return ui.Line(os.date("%m/%d %H:%M", time))
	else
		return ui.Line(os.date("%m/%d  %Y", time))
	end
end

function Linemode:mtime()
	local time = (self._file.cha.modified or 0) // 1
	if time == 0 then
		return ui.Line("")
	elseif os.date("%Y", time) == os.date("%Y") then
		return ui.Line(os.date("%m/%d %H:%M", time))
	else
		return ui.Line(os.date("%m/%d  %Y", time))
	end
end

function Linemode:permissions() return ui.Line(self._file.cha:permissions() or "") end

function Linemode:owner()
	local user = self._file.cha.uid and ya.user_name(self._file.cha.uid) or self._file.cha.uid
	local group = self._file.cha.gid and ya.group_name(self._file.cha.gid) or self._file.cha.gid
	return ui.Line(string.format("%s:%s", user or "-", group or "-"))
end

function Linemode:render()
	local lines = {}
	for _, c in ipairs(self._children) do
		lines[#lines + 1] = (type(c[1]) == "string" and self[c[1]] or c[1])(self)
	end
	return ui.Line(lines)
end

-- Children
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
