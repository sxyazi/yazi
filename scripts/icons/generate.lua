local dark = {
	icons_by_filename = require("default.icons_by_filename"),
	icons_by_file_extension = require("default.icons_by_file_extension"),
}
local light = {
	icons_by_filename = require("light.icons_by_filename"),
	icons_by_file_extension = require("light.icons_by_file_extension"),
}

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
	local dark, light = "", ""
	for _, v in ipairs(list) do
		-- stylua: ignore
		dark = dark .. string.format('\t{ name = "%s", text = "%s", fg = "%s" },\n', v.name, v.text, v.fg_dark)
		light = light .. string.format('\t{ name = "%s", text = "%s", fg = "%s" },\n', v.name, v.text, v.fg_light)
	end
	return dark, light
end

function save(typ, files, exts)
	local p = string.format("../../yazi-config/preset/theme-%s.toml", typ)
	local s = io.open(p, "r"):read("*a")
	s = s:gsub("files = %[\n(.-)\n%]", string.format("files = [\n%s]", files))
	s = s:gsub("exts = %[\n(.-)\n%]", string.format("exts = [\n%s]", exts))
	io.open(p, "w"):write(s)
end

local dark_files, light_files = dump(rearrange("files"))
local dark_exts, light_exts = dump(rearrange("exts"))

save("dark", dark_files, dark_exts)
save("light", light_files, light_exts)
