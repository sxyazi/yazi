local M = {}

local function top(url)
	while url.parent and url.parent.name do
		url = url.parent
	end
	return url.name and url or nil
end

local function entry(url)
	local top = top(url)
	if not top then
		return -- We are at the root, no entry to return
	end
	return fs.trash.entry {
		top = top.spec.domain,
		rel = url:strip_prefix(top) or Path.os(""),
	}
end

local function file(url, ent)
	return File {
		url = url,
		cha = ent.cha,
		link_to = ent.link_to,
		backing = ent.backing,
	}
end

local function files(parent, ents)
	for i, ent in ipairs(ents) do
		local url = parent:join(ent.name):into_domain(ent.key)
		ents[i] = { cha = ent.lcha, file = file(url, ent) }
	end
	return ents
end

local function notify(action, err)
	ya.notify {
		title = "Trash",
		content = string.format("Failed to %s: %s", action, err),
		level = "error",
		timeout = 10,
	}
end

local function absolute(url)
	if url.is_absolute then
		return fs.clean_url(url)
	end

	local cwd, err = fs.cwd()
	if not cwd then
		return nil, err
	end

	local root = cwd.path
	while root.parent do
		root = root.parent
	end
	return fs.clean_url(url:join(root:join(url.path)))
end

local restore_recursively
local function restore(ent)
	local id = { top = ent.top, rel = ent.rel }
	local ok, err = fs.trash.restore { ent }
	if ok then
		return true, 0
	elseif err.kind ~= "AlreadyExists" then
		return false, 1, err
	end
	return restore_recursively(id, err)
end

restore_recursively = function(id, collision)
	local ent, err = fs.trash.entry(id)
	if not ent then
		return false, 1, err
	elseif not ent.cha.is_dir or ent.cha.is_indirect then
		return false, 1, collision
	end

	local target = ent.original and fs.cha(Url(ent.original), false)
	if not target or not target.is_dir or target.is_indirect then
		return false, 1, collision
	end

	local ents, err = fs.trash.list(fs.trash.entry(ent))
	if not ents then
		return false, 1, err
	end

	local failed, changed, first = 0, false, nil
	for _, child in ipairs(ents) do
		local a, b, c = restore(child)
		changed, failed, first = changed or a, failed + b, first or c
	end

	if failed ~= 0 then
		return changed, failed, first
	end

	local ok, err = fs.trash.remove("dir", ent)
	return changed or ok, ok and 0 or 1, err
end

function M:setup()
	ps.sub_remote("trash-restore", function(args)
		ya.async(function()
			local ents, err = {}, nil
			for i, arg in ipairs(args) do
				ents[i], err = entry(Url(arg))
				if not ents[i] then
					return notify("restore", err)
				end
			end

			local changed, failed, first = false, 0, nil
			for _, ent in ipairs(ents) do
				local a, b, c = restore(ent)
				changed, failed, first = changed or a, failed + b, first or c
			end

			if changed then
				ya.emit("refresh", {})
			end
			if failed ~= 0 then
				notify("restore", string.format("%d item(s) could not be restored: %s", failed, first))
			end
		end)
	end)

	ps.sub_remote("trash-empty", function()
		ya.async(function()
			local confirmed = ya.confirm {
				pos = { "center", w = 60, h = 10 },
				title = ui.Line("Empty trash?"):style(th.confirm.title),
				body = ui.Text("All items in the trash will be permanently deleted."),
			}
			if not confirmed then
				return
			end

			local ok, err = fs.trash.empty()
			if ok then
				ya.emit("refresh", {})
			else
				notify("empty", err)
			end
		end)
	end)
end

function M:entry()
	local url, err = absolute(Url("trash:///@/."))
	if url then
		ya.emit("cd", { url })
	else
		notify("open", err)
	end
end

function M:spot(job)
	local ent, err = entry(job.file.url)
	if not ent then
		return ya.err(tostring(err))
	end

	local rows = {
		ui.Row({ "Trash" }):style(ui.Style():fg("green")),
		ui.Row { "  Original:", ui.Text(ent.original and tostring(ent.original) or "-"):wrap(ui.Wrap.YES) },
		ui.Row { "  Backing:", ui.Text(tostring(ent.backing)):wrap(ui.Wrap.YES) },
		ui.Row {},
	}

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

function M:provide(job)
	local op = job.op
	if op == "Absolute" or op == "Canonicalize" then
		return absolute(job.url)
	elseif op == "Casefold" then
		return job.url
	elseif op == "Metadata" or op == "SymlinkMetadata" then
		local ent, err = entry(job.url)
		if ent then
			return fs.trash.metadata(ent, op == "Metadata")
		elseif err then
			return nil, err
		else
			return Cha { mode = tonumber("40700", 8) }
		end
	elseif op == "ReadDir" then
		local ent, err = entry(job.url)
		if err then
			return nil, err
		end
		local ents, err = fs.trash.list(ent)
		if ents then
			return files(job.url, ents)
		else
			return nil, err
		end
	elseif op == "Revalidate" then
		local ent, err = entry(job.file.url)
		if err then
			return nil, err
		else
			return fs.trash.revalidate(ent, job.file)
		end
	elseif op == "File" then
		local ent, err = entry(job.url)
		if ent then
			return file(job.url, ent)
		else
			return nil, err
		end
	elseif op == "RemoveFile" or op == "RemoveDir" then
		local ent, err = entry(job.url)
		if ent then
			return fs.trash.remove(op == "RemoveDir" and "dir" or "file", ent)
		else
			return false, err
		end
	end

	return false, Error.custom("Unsupported trash operation: " .. op)
end

return M
