table.unpack = table.unpack or unpack

ya = ya or {}

function ya.clamp(min, x, max)
	if x < min then
		return min
	elseif x > max then
		return max
	else
		return x
	end
end

function ya.round(x) return x >= 0 and math.floor(x + 0.5) or math.ceil(x - 0.5) end

function ya.list_merge(a, b)
	for _, v in ipairs(b) do
		a[#a + 1] = v
	end
	return a
end

function ya.basename(s) return s:gsub("(.*[/\\])(.*)", "%2") end

function ya.readable_size(size)
	local units = { "B", "K", "M", "G", "T", "P", "E", "Z", "Y", "R", "Q" }
	local i = 1
	while size > 1024.0 and i < #units do
		size = size / 1024.0
		i = i + 1
	end
	return string.format("%.1f%s", size, units[i])
end

function ya.readable_path(path)
	local home = os.getenv("HOME") or os.getenv("USERPROFILE")
	if not home then
		return path
	elseif path:sub(1, #home) == home then
		return "~" .. path:sub(#home + 1)
	else
		return path
	end
end

function ya.child_at(position, children)
	for i = #children, 1, -1 do
		if children[i]._area:contains(position) then
			return children[i]
		end
	end
end
