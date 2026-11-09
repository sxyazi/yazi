local self <close>, acc, f = ...
assert(type(f) == "function", "expected a function")

local is_error = Error.is

local function step(...)
	local value, err = ...
	if err and self._fallible and is_error(err) then
		return nil, err
	elseif value == nil then
		return acc, nil
	end

	local value, err = f(acc, ...)
	if err and self._fallible and is_error(err) then
		return nil, err
	end

	acc = value
	return step(self())
end

return step(self())
