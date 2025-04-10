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

	local cmd = M.with_env()
	if job.args.flatten then
		cmd = cmd:arg("-flatten")
	end

	-- stylua: ignore
	local status, err = cmd:args {
		tostring(job.file.url), "-auto-orient", "-strip",
		"-sample", string.format("%dx%d>", rt.preview.max_width, rt.preview.max_height),
		"-quality", rt.preview.image_quality,
		string.format("JPG:%s", cache),
	}:status()

	if status then
		return status.success
	else
		return true, Err("Failed to start `magick`, error: %s", err)
	end
end

function M:spot(job) require("file"):spot(job) end

function M.with_env()
	local cmd = Command("magick"):env("MAGICK_THREAD_LIMIT", 1)
	if rt.tasks.image_alloc > 0 then
		cmd = cmd:env("MAGICK_MEMORY_LIMIT", rt.tasks.image_alloc):env("MAGICK_DISK_LIMIT", "1MiB")
	end
	return cmd
end

return M
