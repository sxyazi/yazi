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

	local _, err = ya.image_show(cache, job.area)
	ya.preview_widget(job, err and ui.Text(tostring(err)):area(job.area):wrap(ui.Wrap.YES))
end

function M:seek() end

function M:preload(job)
	local cache = ya.file_cache(job)
	if not cache or fs.cha(cache) then
		return true
	end

	-- stylua: ignore
	local cmd = Command("resvg"):arg {
		"-w", rt.preview.max_width, "-h", rt.preview.max_height,
		"--image-rendering", "optimizeSpeed",
		tostring(job.file.url), tostring(cache)
	}
	if rt.tasks.image_alloc > 0 then
		cmd = cmd:memory(rt.tasks.image_alloc)
	end

	local child, err = cmd:spawn()
	if not child then
		return true, Err("Failed to start `resvg`, error: %s", err)
	end

	local status, err
	while true do
		ya.sleep(0.2)

		status, err = child:try_wait()
		if status or err then
			break
		end

		local id, mem = child:id(), nil
		if id then
			mem = ya.proc_info(id).mem_resident
		end
		if mem and mem > rt.tasks.image_alloc then
			child:start_kill()
			err = Err("memory limit exceeded, pid: %s, memory: %s", id, mem)
			break
		end
	end

	if status then
		return status.success
	else
		return true, Err("Error while running `resvg`: %s", err)
	end
end

function M:spot(job) require("file"):spot(job) end

return M
