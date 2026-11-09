local self, f = ...
assert(type(f) == "function", "expected a function")

local found, err = self:any(function(...)
	local keep, err = f(...)
	return not keep, err
end)
if err then
	return false, err
end

return not found, nil
