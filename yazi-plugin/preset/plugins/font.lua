local TEXT = "ABCDEFGHIJKLM\nNOPQRSTUVWXYZ\nabcdefghijklm\nnopqrstuvwxyz\n1234567890\n!$&*()[]{}"

local M = {}

function M:peek(job)
	local start, cache = os.clock(), ya.file_cache(job)
	if not cache or self:preload(job) ~= 1 then
		return
	end

	ya.sleep(math.max(0, PREVIEW.image_delay / 1000 + start - os.clock()))
	ya.image_show(cache, job.area)
	ya.preview_widgets(job, {})
end

function M:seek() end

function M:preload(job)
	local cache = ya.file_cache(job)
	if not cache or fs.cha(cache) then
		return 1
	end

	local status, err = Command("magick"):args({
		"-size",
		"800x560",
		"-gravity",
		"center",
		"-font",
		tostring(job.file.url),
		"-pointsize",
		64,
		"xc:white",
		"-fill",
		"black",
		"-annotate",
		"+0+0",
		TEXT,
		"JPG:" .. tostring(cache),
	}):status()

	if status then
		return status.success and 1 or 2
	else
		ya.err("Failed to start `magick`, error: " .. err)
		return 0
	end
end

return M
