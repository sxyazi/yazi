local M = {}

function M.relative_url(url, line)
	line = line:gsub("[\r\n]+$", "")
	local ok, path = pcall(Path.os, line)
	return ok and not path.is_absolute and url:join(path)
end

function M.file(url)
	local file, err = fs.file(url.physical)
	return file and File { url = url, cha = file.cha, link_to = file.link_to }, err
end

return M
