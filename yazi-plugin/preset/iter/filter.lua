local self, f = ...
assert(type(f) == "function", "expected a function")

local is_error = Error.is

return self:map(function(...)
	local keep, err = f(...)
	if err and self._fallible and is_error(err) then
		return nil, err
	elseif keep then
		return ...
	end
end)
