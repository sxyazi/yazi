local cache = {}

state = setmetatable({
	clear = function() cache[YAZI_PLUGIN_NAME] = nil end,
}, {
	__index = function(_, k)
		local bucket = YAZI_PLUGIN_NAME
		return cache[bucket] and cache[bucket][k]
	end,

	__newindex = function(_, k, v)
		local bucket = YAZI_PLUGIN_NAME
		cache[bucket] = cache[bucket] or {}
		cache[bucket][k] = v
	end,
})
