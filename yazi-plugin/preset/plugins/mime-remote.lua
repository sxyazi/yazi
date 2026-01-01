local M = {}

local function stale_cache(file)
	local url = file.url
	local lock = url.scheme.cache:join(string.format("%%lock/%s", url:hash(true)))

	local f = io.open(tostring(lock), "r")
	if not f then
		return true
	end

	local hash = f:read(32)
	f:close()
	return hash ~= file.cha:hash(true)
end

function M:fetch(job)
	local updates, unknown, state = {}, {}, {}
	for i, file in ipairs(job.files) do
		if file.cha.is_dummy then
			-- Skip dummy files
		elseif not file.cache then
			unknown[#unknown + 1] = file
		elseif not fs.cha(Url(file.cache)) then
			updates[file.url], state[i] = "vfs/absent", true
		elseif stale_cache(file) then
			updates[file.url], state[i] = "vfs/stale", true
		else
			unknown[#unknown + 1] = file
		end
	end

	if next(updates) then
		ya.emit("update_mimes", { updates = updates })
	end

	if #unknown == 0 then
		return state
	else
		return self.fallback_local(job, unknown, state)
	end
end

function M.fallback_local(job, unknown, state)
	local indices = {}
	for i, f in ipairs(job.files) do
		indices[f:hash()] = i
	end

	local result = require("mime.local"):fetch(ya.dict_merge(job, { files = unknown }))
	for i, f in ipairs(unknown) do
		if type(result) == "table" then
			state[indices[f:hash()]] = result[i]
		else
			state[indices[f:hash()]] = result
		end
	end
	return state
end

return M
