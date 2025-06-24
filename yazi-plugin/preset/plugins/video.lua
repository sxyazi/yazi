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

function M:seek(job)
	local h = cx.active.current.hovered
	if h and h.url == job.file.url then
		ya.emit("peek", {
			math.max(0, cx.active.preview.skip + job.units),
			only_if = job.file.url,
		})
	end
end

function M:preload(job)
	local cache = ya.file_cache(job)
	if not cache then
		return true
	end

	local cha = fs.cha(cache)
	if cha and cha.len > 0 then
		return true
	end

	local meta, err = self.list_meta(job.file.url, "format=duration:stream_disposition=attached_pic")
	if not meta then
		return true, err
	elseif not meta.format.duration then
		return true, Err("Failed to get video duration")
	end

	local pic = M.has_pic(meta)
	local percent = (pic and 0 or 5) + job.skip
	if percent > 95 then
		ya.emit("peek", { pic and 95 or 90, only_if = job.file.url, upper_bound = true })
		return false
	end

	-- stylua: ignore
	local cmd = Command("ffmpeg"):arg({
		"-v", "quiet", "-threads", 1, "-hwaccel", "auto",
		"-skip_frame", "nokey",
		"-an", "-sn", "-dn",
	})

	if percent ~= 0 then
		cmd:arg { "-ss", math.floor(meta.format.duration * percent / 100) }
	end
	cmd:arg { "-i", tostring(job.file.url) }
	if percent == 0 then
		cmd:arg { "-map", "disp:attached_pic" }
	end

	-- stylua: ignore
	local status, err = cmd:arg({
		"-vframes", 1,
		"-q:v", 31 - math.floor(rt.preview.image_quality * 0.3),
		"-vf", string.format("scale='min(%d,iw)':'min(%d,ih)':force_original_aspect_ratio=decrease:flags=fast_bilinear", rt.preview.max_width, rt.preview.max_height),
		"-f", "image2",
		"-y", tostring(cache),
	}):status()

	if not status then
		return true, Err("Failed to start `ffmpeg`, error: %s", err)
	elseif not status.success then
		return false, Err("`ffmpeg` exited with error code: %s", status.code)
	else
		return true
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
			:col_style(th.spot.tbl_col)
			:cell_style(th.spot.tbl_cell)
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
	local cmd = Command("ffprobe"):arg { "-v", "quiet" }
	if not entries:find("attached_pic", 1, true) then
		cmd:arg { "-select_streams", "v" }
	end

	local output, err = cmd:arg({ "-show_entries", entries, "-of", "json=c=1", tostring(url) }):output()
	if not output then
		return nil, Err("Failed to start `ffprobe`, error: %s", err)
	end

	local t = ya.json_decode(output.stdout)
	if not t then
		return nil, Err("Failed to decode `ffprobe` output: %s", output.stdout)
	elseif type(t) ~= "table" then
		return nil, Err("Invalid `ffprobe` output: %s", output.stdout)
	end

	t.format = t.format or {}
	t.streams = t.streams or {}
	return t
end

function M.has_pic(meta)
	for _, s in ipairs(meta.streams) do
		if s.disposition and s.disposition.attached_pic == 1 then
			return true
		end
	end
end

return M
