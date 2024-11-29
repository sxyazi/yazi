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

function M:seek(job)
	local h = cx.active.current.hovered
	if h and h.url == job.file.url then
		ya.manager_emit("peek", {
			math.max(0, cx.active.preview.skip + job.units),
			only_if = job.file.url,
		})
	end
end

function M:preload(job)
	local percent = 5 + job.skip
	if percent > 95 then
		ya.manager_emit("peek", { 90, only_if = job.file.url, upper_bound = true })
		return 2
	end

	local cache = ya.file_cache(job)
	if not cache then
		return 1
	end

	local cha = fs.cha(cache)
	if cha and cha.len > 0 then
		return 1
	end

	local meta, err = self.list_meta(job.file.url, "format=duration")
	if not meta then
		ya.err(tostring(err))
		return 0
	elseif not meta.format.duration then
		return 0
	end

	local ss = math.floor(meta.format.duration * percent / 100)
	local qv = 31 - math.floor(PREVIEW.image_quality * 0.3)
	-- stylua: ignore
	local status, err = Command("ffmpeg"):args({
		"-v", "quiet", "-hwaccel", "auto",
		"-skip_frame", "nokey", "-ss", ss,
		"-an", "-sn", "-dn",
		"-i", tostring(job.file.url),
		"-vframes", 1,
		"-q:v", qv,
		"-vf", string.format("scale=%d:-2:flags=fast_bilinear", PREVIEW.max_width),
		"-f", "image2",
		"-y", tostring(cache),
	}):status()

	if status then
		return status.success and 1 or 2
	else
		ya.err("Failed to start `ffmpeg`, error: " .. err)
		return 0
	end
end

function M:spot(job)
	local rows = self:spot_base(job)
	rows[#rows + 1] = ui.Row {}

	ya.spot_table(
		job,
		ui.Table(ya.list_merge(rows, require("file"):spot_base(job)))
			:area(ui.Pos { "center", w = 60, h = 20 })
			:row(1)
			:col(1)
			:col_style(ui.Style():fg("blue"))
			:cell_style(ui.Style():fg("yellow"):reverse())
			:widths { ui.Constraint.Length(14), ui.Constraint.Fill(1) }
	)
end

function M:spot_base(job)
	local meta, err = self.list_meta(job.file.url, "format=duration:stream=codec_name,codec_type,width,height")
	if not meta then
		ya.err(tostring(err))
		return {}
	end

	local dur = meta.format.duration or 0
	local rows = {
		ui.Row({ "Video" }):style(ui.Style():fg("green")),
		ui.Row { "  Duration:", string.format("%d:%02d", math.floor(dur / 60), math.floor(dur % 60)) },
	}

	for i, s in ipairs(meta.streams) do
		if s.codec_type == "video" then
			rows[#rows + 1] = ui.Row { string.format("  Stream %d:", i), "video" }
			rows[#rows + 1] = ui.Row { "    Codec:", s.codec_name }
			rows[#rows + 1] = ui.Row { "    Size:", string.format("%dx%d", s.width, s.height) }
		elseif s.codec_type == "audio" then
			rows[#rows + 1] = ui.Row { string.format("  Stream %d:", i), "audio" }
			rows[#rows + 1] = ui.Row { "    Codec:", s.codec_name }
		end
	end
	return rows
end

function M.list_meta(url, entries)
	local output, err =
		Command("ffprobe"):args({ "-v", "quiet", "-show_entries", entries, "-of", "json=c=1", tostring(url) }):output()
	if not output then
		return nil, Err("Failed to start `ffprobe`, error: " .. err)
	end

	local t = ya.json_decode(output.stdout)
	if not t then
		return nil, Err("Failed to decode `ffprobe` output: " .. output.stdout)
	elseif type(t) ~= "table" then
		return nil, Err("Invalid `ffprobe` output: " .. output.stdout)
	end

	t.format = t.format or {}
	t.streams = t.streams or {}
	return t
end

return M
