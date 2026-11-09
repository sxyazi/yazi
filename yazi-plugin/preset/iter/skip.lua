local self, n = ...
assert(math.type(n) == "integer" and n >= 0, "expected a non-negative integer")

return self:filter(function()
	if n == 0 then
		return true
	end
	n = n - 1
end)
