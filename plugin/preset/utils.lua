utils = utils or {}

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
