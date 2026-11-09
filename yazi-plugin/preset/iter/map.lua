local self, f = ...
assert(type(f) == "function", "expected a function")

local is_error = Error.is
local next = self._next
local apply

local function result(...)
	local value, err = ...
	if value ~= nil or (err and self._fallible and is_error(err)) then
		return ...
	end
	return apply(next())
end

apply = function(...)
	local value, err = ...
	if value == nil or (err and self._fallible and is_error(err)) then
		return ...
	end
	return result(f(...))
end

self._next = function() return apply(next()) end
return self, nil, nil, self
