local dark = require("icons-default")
local light = require("icons-light")

function rearrange(by)
	local map = {}
	local source = by == "exts" and "icons_by_file_extension" or "icons_by_filename"
	for k, v in pairs(dark[source]) do
		map[k] = map[k] or {}
		map[k].icon = v.icon
		map[k].fg_dark = v.color:lower()
	end
	for k, v in pairs(light[source]) do
		map[k].fg_light = v.color:lower()
	end
	return map
end

function dump(map)
	local list = {}
	for k, v in pairs(map) do
		list[#list + 1] = { name = k, text = v.icon, fg_dark = v.fg_dark, fg_light = v.fg_light }
	end
	table.sort(list, function(a, b) return a.name:lower() < b.name:lower() end)
	for _, v in ipairs(list) do
		-- stylua: ignore
		print(string.format('\t{ name = "%s", text = "%s", fg_dark = "%s", fg_light = "%s" },', v.name, v.text, v.fg_dark, v.fg_light))
	end
end

print("files = [")
dump(rearrange("files"))
print("]")

print("exts = [")
dump(rearrange("exts"))
print("]")
