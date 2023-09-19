utils = utils or {}

function utils.basename(str) return string.gsub(str, "(.*[/\\])(.*)", "%2") end
