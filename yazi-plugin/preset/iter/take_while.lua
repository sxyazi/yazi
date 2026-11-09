local self, f = ...
assert(type(f) == "function", "expected a function")

local is_error = Error.is
local next = self._next

local function step(...)
	local value, err = ...
	if err and self._fallible and is_error(err) then
		return nil, err
	elseif value == nil then
		return
	end

	local keep, err = f(...)
	if err and self._fallible and is_error(err) then
		return nil, err
	elseif keep then
		return ...
	end
	next = nil
end

self._next = function()
	if next then
		return step(next())
	end
end
return self, nil, nil, self
