local TEXT = "ABCDEFGHIJKLM\nNOPQRSTUVWXYZ\nabcdefghijklm\nnopqrstuvwxyz\n1234567890\n!$&*()[]{}"

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

	local status, err = Command("magick"):arg({
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
		return status.success
	else
		return true, Err("Failed to start `magick`, error: %s", err)
	end
end

return M
