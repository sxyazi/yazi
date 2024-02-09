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

function ya.flat(t)
	local r = {}
	for _, v in ipairs(t) do
		if type(v) == "table" then
			for _, v2 in ipairs(ya.flat(v)) do
				r[#r + 1] = v2
			end
		else
			r[#r + 1] = v
		end
	end
	return r
end

function ya.sync(f)
	YAZI_SYNC_BLOCKS = YAZI_SYNC_BLOCKS + 1
	if YAZI_SYNC_CALLS == YAZI_SYNC_BLOCKS then
		YAZI_SYNC_ENTRY = f
	end

	local blocks = YAZI_SYNC_BLOCKS
	return function(...) return ya.plugin_retrieve(YAZI_PLUGIN_NAME, blocks, ...) end
end

function ya.basename(str) return string.gsub(str, "(.*[/\\])(.*)", "%2") end

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
	if home == nil then
		return path
	elseif string.sub(path, 1, #home) == home then
		return "~" .. string.sub(path, #home + 1)
	else
		return path
	end
end
