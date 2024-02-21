package.path = BOOT.plugin_dir .. "/?.yazi/init.lua;" .. package.path

local _require = require
require = function(name)
	YAZI_PLUGIN_NAME, YAZI_SYNC_CALLS = name, 0
	return _require(name)
end

YAZI_SYNC_BLOCKS = {}
