local M = {}

function M:peek(job)
	local start, cache = os.clock(), ya.file_cache(job)
	if not cache then
		return
	end

	local ok, err = self:preload(job)
	if not ok or err then
		return
	end

	ya.sleep(math.max(0, rt.preview.image_delay / 1000 + start - os.clock()))
	ya.image_show(cache, job.area)
	ya.preview_widgets(job, {})
end

function M:seek() end

function M:preload(job)
	local cache = ya.file_cache(job)
	if not cache or fs.cha(cache) then
		return true
	end

	-- stylua: ignore
	local cmd = require("magick").with_env():args {
		tostring(job.file.url), "-strip",
		"-resize", string.format("%dx%d^", rt.preview.max_width, rt.preview.max_height),
		"-quality", rt.preview.image_quality,
		string.format("JPG:%s", cache),
	}

	local status, err = cmd:status()
	if status then
		return status.success
	else
		return true, Err("Failed to start `magick`, error: %s", err)
	end
end

function M:spot(job) require("file"):spot(job) end

return M
