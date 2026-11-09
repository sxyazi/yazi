local self = ...

self._kw = true
if self._next ~= self._base then
	return self, nil, nil, self
end

local ty = type(self._source)
assert(ty == "table" or ty == "userdata", "expected a pairs-compatible value")
local iter, state, control, closing = pairs(self._source)

local function update(...)
	control = ...
	return ...
end

self._next = function() return update(iter(state, control)) end
self._base = self._next
self._collect = function(out)
	for key, value in iter, state, control do
		control = key
		out[key] = value
	end
	return out, nil
end

if closing ~= nil then
	self._closing[#self._closing + 1] = closing
end

return self, nil, nil, self
