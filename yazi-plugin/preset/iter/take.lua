local self, n = ...
assert(math.type(n) == "integer" and n >= 0, "expected a non-negative integer")

local next = self._next
self._next = function()
	if n > 0 then
		n = n - 1
		return next()
	end
end

return self, nil, nil, self
