local cache = {}
local sub_mt = {
	__index = function(target, k)
		local bucket = rawget(target, "__yazi_bucket")
		return cache[bucket] and cache[bucket][k]
	end,
	__newindex = function(target, k, v)
		local bucket = rawget(target, "__yazi_bucket")
		cache[bucket] = cache[bucket] or {}
		cache[bucket][k] = v
	end,
}

state = setmetatable({}, {
	__index = function(_, k)
		local bucket = YAZI_PLUGIN_NAME
		return cache[bucket] and cache[bucket][k]
	end,
	__newindex = function(_, k, v)
		local bucket = YAZI_PLUGIN_NAME
		cache[bucket] = cache[bucket] or {}
		cache[bucket][k] = v
	end,
	__call = function() return setmetatable({ __yazi_bucket = YAZI_PLUGIN_NAME }, sub_mt) end,
})
