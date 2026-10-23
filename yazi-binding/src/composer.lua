local get, set = ...
local type, error = type, error
local cache = {}

return setmetatable({}, {
	__metatable = false,
	__index = function(_, key)
		local value = cache[key]
		if value == cache then
			return nil
		elseif value ~= nil then
			return value
		elseif type(key) == "string" then
			value = get(key)
			cache[key] = value == nil and cache or value
		else
			error("expected string")
		end
		return value
	end,
	__newindex = function(_, key, value)
		if type(key) ~= "string" then
			error("expected string")
		elseif set then
			value = set(key, value)
		end
		cache[key] = value
	end,
})
