local load, ud_callable = ...
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

local function index(source, i) return source[i] end

return function(source, state, control, closing)
	local ty = type(source)
	local mt = ty == "table" and getmetatable(source)

	local callable = false
	if ty == "userdata" then
		callable = ud_callable(source)
	elseif type(mt) == "table" then
		callable = rawget(mt, "__call") ~= nil
	end

	local collect, next = false
	if ty == "table" and not callable then
		next, collect = Iter._array(source, rawget)
	elseif ty == "userdata" and not callable then
		next, collect = Iter._array(source, index)
	elseif source == nil then
		next = function()
			local err = state
			state = nil
			return nil, err
		end
	else
		assert(ty == "function" or callable, "expected an array or iterator")
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
		_kv = false,
		_collect = collect,
		_fallible = false,
		_closing = { closing },
	}, Iter)
	return me, nil, nil, me
end
