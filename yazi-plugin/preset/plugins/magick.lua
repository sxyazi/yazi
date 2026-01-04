local M = {}

function M:peek(job)
	local start, cache = os.clock(), ya.file_cache(job)
	if not cache then
		return
	end

	local ok, err = self:preload(job)
	if not ok or err then
		return ya.preview_widget(job, err)
	end

	ya.sleep(math.max(0, rt.preview.image_delay / 1000 + start - os.clock()))

	local _, err = ya.image_show(cache, job.area)
	ya.preview_widget(job, err)
end

function M:seek() end

function M:preload(job)
	local cache = ya.file_cache(job)
	if not cache or fs.cha(cache) then
		return true
	end

	local cmd = M.with_limit():arg(tostring(job.file.path))
	if job.args.flatten then
		cmd:arg("-flatten")
	end
	cmd:arg { "-auto-orient", "-strip" }

	local size = string.format("%dx%d>", rt.preview.max_width, rt.preview.max_height)
	if rt.preview.image_filter == "nearest" then
		cmd:arg { "-sample", size }
	elseif rt.preview.image_filter == "catmull-rom" then
		cmd:arg { "-filter", "catrom", "-thumbnail", size }
	elseif rt.preview.image_filter == "lanczos3" then
		cmd:arg { "-filter", "lanczos", "-thumbnail", size }
	elseif rt.preview.image_filter == "gaussian" then
		cmd:arg { "-filter", "gaussian", "-thumbnail", size }
	else
		cmd:arg { "-filter", "triangle", "-thumbnail", size }
	end

	cmd:arg { "-quality", rt.preview.image_quality }
	if job.args.bg then
		cmd:arg { "-background", job.args.bg, "-alpha", "remove" }
	end

	local status, err = cmd:arg(string.format("JPG:%s", cache)):status()
	if not status then
		return true, Err("Failed to start `magick`, error: %s", err)
	elseif not status.success then
		return false, Err("`magick` exited with error code: %s", status.code)
	else
		return true
	end
end

function M:spot(job) require("file"):spot(job) end

function M.with_limit()
	local cmd = Command("magick"):arg { "-limit", "thread", 1 }
	if rt.tasks.image_alloc > 0 then
		cmd:arg { "-limit", "memory", rt.tasks.image_alloc, "-limit", "disk", "1MiB" }
	end
	if rt.tasks.image_bound[1] > 0 then
		cmd:arg { "-limit", "width", rt.tasks.image_bound[1] }
	end
	if rt.tasks.image_bound[2] > 0 then
		cmd:arg { "-limit", "height", rt.tasks.image_bound[2] }
	end
	return cmd
end

return M
