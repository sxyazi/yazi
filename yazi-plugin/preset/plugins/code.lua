local M = {}

local function get_search_occurrences(url)
	if not url.is_search then
		return
	end
	local occurrences = tostring(url):match("^search://(.-)//")
	local lines = {}
	for occurrence in occurrences:gmatch("[^,]+") do
		local line, col = occurrence:match("(%d+)-(%d+)")
		table.insert(lines, { tonumber(line), tonumber(col) })
	end

	if #lines == 0 then
		return
	end

	return lines
end

-- local function get_next_occurrence(occurrences, current_line, direction)
-- 	local step = direction == "up" and -1 or 1
-- 	local start = direction == "up" and #occurrences or 1
-- 	local stop = direction == "up" and 1 or #occurrences
--
-- 	for i = start, stop, step do
-- 		local line = occurrences[i]
-- 		if go_up and line >= current_line then
-- 			return line
-- 		end
-- 		if direction == "down" and line <= current_line then
-- 			return line
-- 		end
--
-- 		-- TODO: Handle return to first element or last one
-- 	end
-- end

function M:peek(job)
	local search_occurrences = get_search_occurrences(job.file.url)
	local search_idx = cx.active.preview.search_idx
	ya.dbg(" search index", search_idx)
	local occurrence
	if search_occurrences and search_idx then
		occurrence = search_occurrences[search_idx]
	end

	ya.dgb("Peek Occurrence:", occurrence)

	-- Todo: Pass the occurrence for highlighting
	local err, bound = ya.preview_code(job)
	if bound then
		ya.emit("peek", { bound, only_if = job.file.url, upper_bound = true })
	elseif err and not err:find("cancelled", 1, true) then
		require("empty").msg(job, err)
	end
end

function M:seek(job)
	local direction = job.units > 0 and "down" or "up"

	local search_idx = cx.active.preview.search_idx
	ya.dbg(" search index", search_idx)

	local h = cx.active.current.hovered
	if not h or h.url ~= job.file.url then
		return
	end

	local search_occurrences = get_search_occurrences(job.file.url)
	if search_occurrences then
		search_idx = (search_idx or 0) + 1
		-- local current_line = cx.active.preview.skip + 1
		-- ya.dbg("Current line:", current_line)
		-- local next_occurrence = get_next_occurrence(search_occurrences, current_line, direction)
		-- for _, occurrence in ipairs(search_occurrences) do
		-- 	local search_line = occurrence[1]
		-- 	-- local col = occurrence[2]
		--
		-- 	if search_line >= current_line then
		-- 		ya.emit("peek", { math.max(0, search_line - 1), only_if = job.file.url })
		-- 		return
		-- 	end
		-- end
	end

	-- ya.emit("peek", { math.max(0, line - 1), only_if = job.file.url })

	local step = math.floor(job.units * job.area.h / 10)
	step = step == 0 and ya.clamp(-1, job.units, 1) or step

	ya.emit("peek", {
		math.max(0, cx.active.preview.skip + step),
		only_if = job.file.url,
		search_idx,
	})
end

function M:spot(job) require("file"):spot(job) end

return M
