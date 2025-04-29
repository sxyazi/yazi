function Err(s, ...) return Error.custom(string.format(s, ...)) end

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

function ya.dict_merge(a, b)
	for k, v in pairs(b) do
		a[k] = v
	end
	return a
end

function ya.readable_size(size)
	local units = { "B", "K", "M", "G", "T", "P", "E", "Z", "Y", "R", "Q" }
	local i = 1
	while size > 1024 and i < #units do
		size = size / 1024
		i = i + 1
	end
	local s = string.format("%.1f%s", size, units[i]):gsub("[.,]0", "", 1)
	return s
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

function ya.child_at(pos, children)
	for i = #children, 1, -1 do
		if children[i]._area:contains(pos) then
			return children[i]
		end
	end
end
