local self = ...

local i = 0
return self:map(function(...)
	i = i + 1
	return i, ...
end)
