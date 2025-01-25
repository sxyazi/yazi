Linemode = {
	_inc = 1000,
	_children = {
		{ "solo", id = 1, order = 1000 },
		{ "spacer", id = 2, order = 2000 },
	},
}

function Linemode:new(file) return setmetatable({ _file = file }, { __index = self }) end

function Linemode:spacer() return " " end

function Linemode:solo()
	local mode = cx.active.pref.linemode
	if mode == "none" or mode == "solo" then
		return ""
	elseif not self[mode] then
		return " " .. mode
	else
		local line = ui.Line(self[mode](self))
		return line:visible() and ui.Line { " ", line } or line
	end
end

function Linemode:size()
	local size = self._file:size()
	if size then
		return ya.readable_size(size)
	else
		local folder = cx.active:history(self._file.url)
		return folder and tostring(#folder.files) or ""
	end
end

function Linemode:btime()
	local time = math.floor(self._file.cha.btime or 0)
	if time == 0 then
		return ""
	elseif os.date("%Y", time) == os.date("%Y") then
		return os.date("%m/%d %H:%M", time)
	else
		return os.date("%m/%d  %Y", time)
	end
end

function Linemode:mtime()
	local time = math.floor(self._file.cha.mtime or 0)
	if time == 0 then
		return ""
	elseif os.date("%Y", time) == os.date("%Y") then
		return os.date("%m/%d %H:%M", time)
	else
		return os.date("%m/%d  %Y", time)
	end
end

function Linemode:permissions() return self._file.cha:perm() or "" end

function Linemode:owner()
	local user = self._file.cha.uid and ya.user_name(self._file.cha.uid) or self._file.cha.uid
	local group = self._file.cha.gid and ya.group_name(self._file.cha.gid) or self._file.cha.gid
	return string.format("%s:%s", user or "-", group or "-")
end

function Linemode:redraw()
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
