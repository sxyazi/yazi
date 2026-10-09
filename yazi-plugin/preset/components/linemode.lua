Linemode = {
	_inc = 1000,
	_children = {
		{ "solo", id = 1, order = 1000 },
		{ "padding", id = 2, order = 2000 },
	},
}

function Linemode:new(file) return setmetatable({ _file = file }, { __index = self }) end

function Linemode:solo()
	if not self._file.in_current then
		return ""
	end

	local mode = cx.tab.pref.linemode
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
		local folder = cx.tab:history(self._file.url)
		return folder and tostring(#folder.files) or ""
	end
end

function Linemode:btime()
	local btime = self._file.stat.btime
	if not btime then
		return ""
	elseif btime.year == ya.time().year then
		return btime:format { month = "2-digit", day = "2-digit", hour24 = "2-digit", minute = "2-digit" }
	else
		return btime:format { month = "2-digit", day = "2-digit" } .. "  " .. btime.year
	end
end

function Linemode:mtime()
	local mtime = self._file.stat.mtime
	if not mtime then
		return ""
	elseif mtime.year == ya.time().year then
		return mtime:format { month = "2-digit", day = "2-digit", hour24 = "2-digit", minute = "2-digit" }
	else
		return mtime:format { month = "2-digit", day = "2-digit" } .. "  " .. mtime.year
	end
end

function Linemode:permissions() return self._file.stat:perm() or "" end

function Linemode:owner()
	local user = ya.user_name and ya.user_name(self._file.stat.uid) or self._file.stat.uid
	local group = ya.group_name and ya.group_name(self._file.stat.gid) or self._file.stat.gid
	return string.format("%s:%s", user, group)
end

function Linemode:padding()
	if not self._file.is_hovered then
		return " "
	end

	local style = Entity:new(self._file):style_rev()
	if style then
		return ui.Span(th.indicator.padding.close):style(style)
	else
		return " "
	end
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
