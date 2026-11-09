local load = ...
local is_error = Error.is
local Iter = setmetatable({}, {
	__index = function(methods, key)
		local method = load(key)
		rawset(methods, key, method)
		return method
	end,
})
Iter.__index = Iter

function Iter:_close(i)
	local _ <close> = self._closing[i]
	self._closing[i] = nil
	if self._closing[i + 1] ~= nil then
		self:_close(i + 1)
	end
end

function Iter:__close()
	if self._closing[1] ~= nil then
		self:_close(1)
	end
end

local function advance(self, ...)
	local _, err = ...
	if err and self._fallible and is_error(err) then
		return false, err
	end
	return ...
end

function Iter:__call() return advance(self, self._next()) end

local function array(source)
	local i = 0
	local function next()
		i = i + 1
		return rawget(source, i)
	end

	local function collect(out)
		if i == 0 then
			table.move(source, 1, #source, #out + 1, out)
			i = #source
			return out, nil
		end

		while true do
			i = i + 1
			local value = rawget(source, i)
			if value == nil then
				return out, nil
			end
			out[#out + 1] = value
		end
	end

	return next, collect
end

return function(source, state, control, closing)
	local ty = type(source)
	local mt = ty == "table" and getmetatable(source)
	local callable = type(mt) == "table" and rawget(mt, "__call") ~= nil

	local collect, next = false
	if ty == "table" and not callable then
		next, collect = array(source)
	elseif source == nil then
		next = function()
			local err = state
			state = nil
			return nil, err
		end
	else
		assert(ty == "function" or ty == "userdata" or callable, "expected an array or iterator")
		next = source
	end

	local function update(...)
		control = ...
		return ...
	end

	local pull = function() return update(next(state, control)) end
	local me = setmetatable({
		_next = pull,
		_base = pull,
		_source = source,
		_kw = false,
		_collect = collect,
		_fallible = false,
		_closing = { closing },
	}, Iter)
	return me, nil, nil, me
end
