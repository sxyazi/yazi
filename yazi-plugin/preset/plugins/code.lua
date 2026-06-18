local M = {}

local function get_search_occurrences(url)
	if not url.is_search then
		return
	end
	-- Shoud we encode ~ too?
	local subject_len, occurrences = tostring(url):match("^search://([^~]+)~(.-)//")
	local lines = {}
	for occurrence in occurrences:gmatch("[^,]+") do
		local line, col = occurrence:match("(%d+)-(%d+)")
		table.insert(lines, { tonumber(line), tonumber(col) })
	end

	if #lines == 0 then
		return
	end

	return tonumber(subject_len), lines
end

local function get_next_occurrence_idx(search_idx, direction, occurrences_len)
	local index

	if direction == "up" then
		index = (search_idx or 1) - 1
		if index < 1 then
			index = occurrences_len
		end
	else
		index = (search_idx or 1) + 1
		if index > occurrences_len then
			index = 1
		end
	end

	return index
end

function M:peek(job)
	local subject_len, search_occurrences = get_search_occurrences(job.file.url)
	local search_idx = job.search_idx

	if search_idx == nil and search_occurrences then
		local occurrence = search_occurrences[1]
		local line = occurrence[1]
		ya.emit("peek", {
			math.max(0, line - 1),
			only_if = job.file.url,
			search_idx = 1,
		})
		return
	end

	local occurrence
	if search_occurrences and search_idx then
		occurrence = search_occurrences[search_idx]
		job.position = {
			line = occurrence[1],
			col = occurrence[2],
			length = subject_len,
		}
	end

	local err, bound = ya.preview_code(job)
	if bound then
		ya.emit("peek", { bound, only_if = job.file.url, upper_bound = true, search_idx = search_idx })
	elseif err and not err:find("cancelled", 1, true) then
		require("empty").msg(job, err)
	end
end

function M:seek(job)
	local direction = job.units > 0 and "down" or "up"
	local search_idx = cx.active.preview.search_idx

	local h = cx.active.current.hovered
	if not h or h.url ~= job.file.url then
		return
	end

	local _, search_occurrences = get_search_occurrences(job.file.url)
	if search_occurrences then
		local next_occurrence_idx = get_next_occurrence_idx(search_idx, direction, #search_occurrences)
		local occurrence = search_occurrences[next_occurrence_idx]
		local line = occurrence[1]

		ya.emit("peek", { math.max(0, line - 1), only_if = job.file.url, search_idx = next_occurrence_idx })
		return
	end

	local step = math.floor(job.units * job.area.h / 10)
	step = step == 0 and ya.clamp(-1, job.units, 1) or step

	ya.emit("peek", {
		math.max(0, cx.active.preview.skip + step),
		only_if = job.file.url,
		search_idx = search_idx,
	})
end

function M:spot(job) require("file"):spot(job) end

return M
