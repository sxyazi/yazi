local self <close>, f = ...
assert(type(f) == "function", "expected a function")

local ok, err = self:fold(true, function(_, ...)
	local _, err = f(...)
	return true, err
end)
return ok or false, err
