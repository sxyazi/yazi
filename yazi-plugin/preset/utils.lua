table.unpack = table.unpack or unpack

utils = utils or {}

function utils.flat(t)
	local r = {}
	for _, v in ipairs(t) do
		if type(v) == "table" then
			for _, v2 in ipairs(utils.flat(v)) do
				r[#r + 1] = v2
			end
		else
			r[#r + 1] = v
		end
	end
	return r
end

function utils.basename(str) return string.gsub(str, "(.*[/\\])(.*)", "%2") end

function utils.readable_size(size)
	local units = { "B", "KB", "MB", "GB", "TB", "PB", "EB" }
	local i = 1
	while size > 1024.0 and i < #units do
		size = size / 1024.0
		i = i + 1
	end
	return string.format("%.1f %s", size, units[i])
end

function utils.readable_path(path)
	local home = os.getenv("HOME") or os.getenv("USERPROFILE")
	if home == nil then
		return path
	elseif string.sub(path, 1, #home) == home then
		return "~" .. string.sub(path, #home + 1)
	else
		return path
	end
end
