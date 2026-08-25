local MAX = ya.input_history_max()

local M = {}

local function state_path()
	-- Follows state logic in yazi-fs/src/xdg.rs
	if ya.target_family() == "windows" then
		-- As per rust Rust dirs crate
		return (os.getenv("APPDATA") or "") .. "/yazi/state/history.json"
	end
	return (os.getenv("XDG_STATE_HOME") or ((os.getenv("HOME") or "") .. "/.local/state")) .. "/yazi/history.json"
end

local function remember(entries, group, value)
	local list = entries[group] or {}
	for i = #list, 1, -1 do
		if list[i] == value then
			table.remove(list, i)
		end
	end
	list[#list+1] = value
	while #list > MAX do
		-- `while` is defensive programming, should never happen
		table.remove(list, 1)
	end
	entries[group] = list
end

-- Needs a `require("history"):setup()` in ~/.config/yazi/init.lua to work for any/all plugins using this
function M:setup(_)
	local path = state_path()
	local entries = {}

	ya.async(function()
		local f = io.open(path, "r")
		if not f then return end -- Return early if file doesn't exist

		local raw = f:read("*a")
		f:close()

		local json = ya.json_decode(raw)
		if not json then
			return nil, Err("Failed to decode history file %s: %s", path, raw)
		elseif type(json) ~= "table" then
			return nil, Err("Invalid history file %s: %s", path, raw)
		end

		for group, list in pairs(json) do
			entries[group] = entries[group] or list
		end

		if next(json) then
			ya.emit("load_history", { json })
		end
	end)

	ps.sub("history", function(body)
		remember(entries, body.group, body.value)

		ya.async(function()
			fs.create("dir_all", Url(path).parent)
			local f = io.open(path, "w")
			if not f then
				return nil, Err("Failed to open history file %s for writing", path)
			end
			f:write(ya.json_encode(entries))
			f:close()
		end)
	end)
end

return M
