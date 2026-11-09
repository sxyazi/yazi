local self, f = ...
self:map(f)

local is_error = Error.is
local iter = ya.iter
local next = self._next
local inner, pull

local function begin(...)
	local value, err = ...
	if err and self._fallible and is_error(err) then
		return nil, err
	elseif value == nil then
		return
	end

	local it = iter(...)
	self._closing[#self._closing + 1] = it
	inner = it._next
	return pull()
end

local function step(...)
	local value, err = ...
	if value ~= nil or (err and self._fallible and is_error(err)) then
		return ...
	end
	inner = nil
	self:_close(#self._closing)
	return pull()
end

pull = function()
	if inner then
		return step(inner())
	end
	return begin(next())
end

self._next = pull
return self, nil, nil, self
