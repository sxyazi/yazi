local self, f = ...

local value, err = self:find(f)
return value ~= nil, value == nil and err or nil
