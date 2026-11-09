local self, f = ...
assert(type(f) == "function", "expected a function")

local is_error = Error.is

local function find(...)
	local value, err = ...
	if err and self._fallible and is_error(err) then
		return nil, err
	elseif value == nil then
		return nil
	end

	local keep, err = f(...)
	if err and self._fallible and is_error(err) then
		return nil, err
	elseif keep then
		return ...
	end
	return find(self())
end

return find(self())
