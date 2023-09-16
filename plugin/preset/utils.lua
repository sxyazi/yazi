yazi = yazi or {}

function yazi.basename(str) return string.gsub(str, "(.*[/\\])(.*)", "%2") end
