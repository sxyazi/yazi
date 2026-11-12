local source, get = ...
local i = 0

local function next()
	i = i + 1
	return get(source, i)
end

local function collect(out)
	if i == 0 and type(source) == "table" then
		table.move(source, 1, #source, #out + 1, out)
		i = #source
		return out, nil
	end

	while true do
		i = i + 1
		local value = get(source, i)
		if value == nil then
			return out, nil
		end
		out[#out + 1] = value
	end
end

return next, collect
