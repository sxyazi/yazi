--- @sync peek

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

local function backing(url)
	local name = url.name
	if not name or Path.os(name).name ~= name then
		return nil, Err("Invalid trash entry name")
	end

	local ent, err = entry(url.parent)
	if err then
		return nil, err
	elseif not ent then
		ent, err = entry(url) -- Trash root has no entry of its own, get backing path from child target
		return ent and Url(ent.backing), err
	end

	local stat, err = ent:lstat()
	if not stat then
		return nil, err
	elseif not stat.is_dir or stat.is_indirect then
		return nil, Err("Trash parent is not a directory")
	end

	return Url(ent.backing):join(name)
end

local function copy(from, to, attrs)
	if from.path == to.path then
		return nil, Err("Cannot copy an item onto itself")
	end

	local stream <close>, err = fs.copy_stream(from, to, attrs)
	if not stream then
		return nil, err
	end
	for n, err in stream do
		if not n then
			return nil, err
		end
		coroutine.yield(n)
	end
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

local remove_recursively
local function remove(ent)
	if not ent then
		return fs.trash.empty()
	end

	local stat, err = ent:stat()
	if not stat then
		return false, err
	elseif stat.is_dir and not stat.is_indirect then
		return remove_recursively(ent)
	else
		return fs.trash.remove("file", ent)
	end
end

remove_recursively = function(ent)
	local ents, err = fs.trash.list(ent)
	if not ents then
		return false, err
	end
	for _, child in ipairs(ents) do
		local ok, err = remove(child)
		if not ok then
			return false, err
		end
	end
	return fs.trash.remove("dir", ent)
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
	end

	local stat, err = ent:stat()
	if not stat then
		return false, 1, err
	elseif not stat.is_dir or stat.is_indirect then
		return false, 1, collision
	end

	local target = ent.original and fs.lstat(Url(ent.original))
	if not target or not target.is_dir or target.is_indirect then
		return false, 1, collision
	end

	local ents, err = fs.trash.list(ya.clone(ent))
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

local function match(job, rules)
	local mime = job.mime:match("^trash/(.+)")
	if not mime then
		return
	end
	return rules:match { file = job.file, mime = mime }(), mime
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

function M:peek(job)
	local rule, mime = match(job, rt.plugin.previewers)
	if not rule then
		return
	end

	job.mime, job.args = mime, rule.run.args
	job.scope:spawn(function()
		local chunk, err = ya.chunk(rule.run.name)
		if not chunk then
			ya.err(err)
		elseif chunk.sync_peek then
			ya.emit("plugin", { rule.run.name, job, method = "peek", scope = ya.clone(job.scope) })
		else
			job.scope:spawn_blocking(function(cx) require(cx.name):peek(cx.job) end, { name = rule.run.name, job = job })
		end
	end)
end

function M:seek(job)
	local rule, mime = match(job, rt.plugin.previewers)
	if not rule then
		return
	end

	job.mime, job.args = mime, rule.run.args
	ya.emit("plugin", { rule.run.name, job, method = "seek", scope = ya.clone(job.scope) })
end

function M:preload(job)
	local rule, mime = match(job, rt.plugin.preloaders)
	if not rule then
		return true
	end

	job.mime, job.args = mime, rule.run.args
	return require(rule.run.name):preload(job)
end

function M:spot(job)
	local ent, err = entry(job.file.url)
	if not ent then
		return ya.err(err)
	end

	local rows = {
		ui.Row({ "Trash" }):style(ui.Style():fg("green")),
		ui.Row { "  Original:", ui.Text(ent.original and tostring(ent.original) or "-"):wrap(ui.Wrap.YES) },
		ui.Row { "  Backing:", ui.Text(tostring(ent.backing)):wrap(ui.Wrap.YES) },
		ui.Row {},
	}

	ya.spot_table(
		job,
		ui.Table(ya.iter(self:spot_base(job)):into(rows))
			:area(ui.Pos { "center", w = 60, h = 20 })
			:row(1)
			:col(1)
			:col_style(th.spot.tbl_col)
			:cell_style(th.spot.tbl_cell)
			:widths { ui.Constraint.Length(14), ui.Constraint.Fill(1) }
	)
end

function M:spot_base(job)
	local stat, pair = job.file.stat, { file = job.file, mime = job.mime }
	local proxy = { file = job.file, mime = job.mime:match("^trash/(.+)") or job.mime }

	local spotter = rt.plugin.spotters:match(pair)()
	local previewer = rt.plugin.previewers:match(proxy)()

	local fetchers = ya.iter(rt.plugin.fetchers:match(pair)):map(function(v) return v.run.name end):into()
	fetchers = #fetchers ~= 0 and fetchers or { "-" }

	local preloaders = ya.iter(rt.plugin.preloaders:match(proxy))
		:map(function(v) return string.format("%s (via trash)", v.run.name) end)
		:into()
	preloaders = #preloaders ~= 0 and preloaders or { "-" }

	return {
		ui.Row({ "Base" }):style(ui.Style():fg("green")),
		ui.Row { "  Created:", stat.btime and stat.btime:format("%c") or "-" },
		ui.Row { "  Modified:", stat.mtime and stat.mtime:format("%c") or "-" },
		ui.Row { "  Mimetype:", job.mime },
		ui.Row {},

		ui.Row({ "Plugins" }):style(ui.Style():fg("green")),
		ui.Row { "  Spotter:", spotter and spotter.run.name or "-" },
		ui.Row { "  Previewer:", previewer and string.format("%s (via trash)", previewer.run.name) or "-" },
		ui.Row({ "  Fetchers:", fetchers }):height(#fetchers),
		ui.Row({ "  Preloaders:", preloaders }):height(#preloaders),
	}
end

function M:provide(job)
	local op = job.op
	if op == "Capabilities" then
		return {
			copy_from = 1,
			copy_to = 1,
			create_dir = 1,
			create_file_new = 1,
			prepare = 1,
			remove_dir_all = 1,
			rename = 1,
			set_attrs = 1,
			symlink_dir = 1,
			symlink_file = 1,
		}
	elseif op == "Absolute" or op == "Canonicalize" then
		return absolute(job.url)
	elseif op == "Casefold" then
		return job.url
	elseif op == "Prepare" then
		if not job.to.parent or job.to.parent.name then
			return -- Only prepare new entries directly under the trash root
		end

		local original
		if job.from.spec.is_local then
			original = job.from.path
		elseif job.from.spec.scheme == job.to.spec.scheme then
			local ent, err = entry(job.from)
			if not ent then
				return nil, err or Err("Cannot copy trash root")
			end
			original = ent.original
		else
			return nil, Error.fs { kind = "Unsupported", message = "Only local or trash sources are supported" }
		end

		local ent, err = fs.trash.create(job.to.name, job.stat, original)
		return ent and job.to:with_domain(ent.key), err
	elseif op == "CopyTo" then
		return ya.co(function()
			local url, err
			if job.to.spec.is_local then
				url = job.to
			elseif job.to.spec.scheme == job.from.spec.scheme then
				url, err = backing(job.to)
			else
				err = Error.fs { kind = "Unsupported", message = "Only local or trash copy destinations are supported" }
			end
			if not url then
				return nil, err
			end

			local ent, err = entry(job.from)
			if not ent then
				return nil, err or Err("Cannot copy trash root")
			end
			return copy(Url(ent.backing), url, job.attrs)
		end)
	elseif op == "CopyFrom" then
		return ya.co(function()
			if not job.from.spec.is_local then
				return nil, Error.fs { kind = "Unsupported", message = "Only local copy sources are supported" }
			end

			local url, err = backing(job.to)
			if not url then
				return nil, err
			end
			return copy(job.from, url, job.attrs)
		end)
	elseif op == "CreateDir" then
		local url, err = backing(job.url)
		if url then
			return fs.create_dir(url)
		else
			return false, err
		end
	elseif op == "CreateFileNew" then
		local url, err = backing(job.url)
		if not url then
			return nil, err
		end

		local handle, err = fs.create_file_new(url)
		if not handle then
			return nil, err
		end
		return { id = 0, random = "none", state = handle:into_shared() }
	elseif op == "Write" then
		return ya.co(function()
			for chunk in job.stream do
				local ok, err = job.state:write_all_at(chunk.from, chunk.bytes)
				if not ok then
					return nil, err
				end
				coroutine.yield(chunk.to)
			end
		end)
	elseif op == "Close" then
		return true
	elseif op == "Lstat" then
		local ent, err = entry(job.url)
		if ent then
			return ent:lstat()
		elseif err then
			return nil, err
		else
			return Stat { mode = tonumber("40700", 8) }
		end
	elseif op == "Stat" then
		local ent, err = entry(job.url)
		if ent then
			return ent:stat()
		elseif err then
			return nil, err
		else
			return Stat { mode = tonumber("40700", 8) }
		end
	elseif op == "ReadDir" then
		return ya.co(function()
			local ent, err = entry(job.url)
			if err then
				return nil, err
			end
			local ents, err = fs.trash.list(ent)
			if not ents then
				return nil, err
			end
			for _, ent in ipairs(ents) do
				local url = job.url:join(ent.name):with_domain(ent.key)
				coroutine.yield(ent:file(url) or ent:dummy(url))
			end
		end)
	elseif op == "Revalidate" then
		local ent, err = entry(job.file.url)
		if err then
			return nil, err
		else
			return fs.trash.revalidate(ent, job.file)
		end
	elseif op == "File" then
		if job.url.spec.domain == "" then
			return nil, Error.fs { kind = "NotFound", message = "Trash target has no key" }
		end

		local ent, err = entry(job.url)
		if ent then
			return ent:file(job.url)
		else
			return nil, err
		end
	elseif op == "SetAttrs" then
		local ent, err = entry(job.url)
		if ent then
			return fs.set_attrs(Url(ent.backing), job.attrs)
		else
			return false, err or Err("Cannot change trash root attributes")
		end
	elseif op == "SymlinkDir" or op == "SymlinkFile" then
		local url, err = backing(job.url)
		if not url then
			return false, err
		end

		local link = op == "SymlinkDir" and fs.symlink_dir or fs.symlink_file
		return link(url, job.original, job.force)
	elseif op == "Rename" then
		local ent, err = entry(job.from)
		if not ent then
			return nil, err or Err("Cannot rename trash root")
		elseif not ent.rel.name and job.from.parent ~= job.to.parent then
			return nil, Error.fs { kind = "Unsupported", message = "Top-level trash items can only be renamed in place" }
		elseif not ent.rel.name then
			local name = job.to.name
			if ent.original then
				local ok, err = fs.trash.set_original(ent, ent.original.parent:join(name))
				return ok and job.to:with_domain(ent.key) or nil, err
			end

			local lstat, err = ent:lstat()
			if not lstat then
				return nil, err
			elseif name == ent.name then
				return job.from
			end

			local target, err = fs.trash.create(name, lstat, Path.os(""))
			if not target then
				return nil, err
			end

			local ok, err = fs.trash.rename(ent, target.backing)
			return ok and job.to.parent:join(target.name):with_domain(target.key) or nil, err
		end

		local url, err = backing(job.to)
		if not url then
			return nil, err
		end

		local ok, err = fs.trash.rename(ent, url.path)
		if not ok then
			return nil, err
		end

		ent, err = entry(job.to)
		return ent and job.to:with_domain(ent.key), err
	elseif op == "RemoveDirAll" then
		return ya.co(function()
			local ent, err = entry(job.url)
			if err then
				return nil, err
			end

			local ok, err = remove(ent)
			if not ok then
				return nil, err
			end
		end)
	elseif op == "RemoveFile" or op == "RemoveDir" then
		local ent, err = entry(job.url)
		if ent then
			return fs.trash.remove(op == "RemoveDir" and "dir" or "file", ent)
		else
			return false, err or Err("Cannot remove trash root")
		end
	end

	return false, Err("Unsupported trash operation: %s", op)
end

return M
