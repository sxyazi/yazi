local self <close>, out = ...
local is_error = Error.is

if out == nil then
	out = {}
else
	assert(type(out) == "table", "expected a table")
end

if self._collect and self._next == self._base and (not self._kw or not self._fallible) then
	return self._collect(out)
end

if self._kw then
	while true do
		local key, value = self()
		if value and self._fallible and is_error(value) then
			return nil, value
		elseif key == nil then
			return out, nil
		end
		out[key] = value
	end
end

while true do
	local value, err = self()
	if err and self._fallible and is_error(err) then
		return nil, err
	elseif value == nil then
		return out, nil
	end
	out[#out + 1] = value
end
