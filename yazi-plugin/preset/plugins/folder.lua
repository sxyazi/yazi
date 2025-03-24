--- @sync peek

local M = {}

function M:peek(job)
	local folder = cx.active.preview.folder
	if not folder or folder.cwd ~= job.file.url then
		return
	end

	local bound = math.max(0, #folder.files - job.area.h)
	if job.skip > bound then
		return ya.mgr_emit("peek", { bound, only_if = job.file.url, upper_bound = true })
	end

	if #folder.files == 0 then
		local done, err = folder.stage()
		local s = not done and "Loading..." or not err and "No items" or string.format("Error: %s", err)
		return ya.preview_widgets(job, {
			ui.Text(s):area(job.area):align(ui.Text.CENTER),
		})
	end

	local entities = {}
	for _, f in ipairs(folder.window) do
		entities[#entities + 1] = Entity:new(f):redraw()
	end

	ya.preview_widgets(job, {
		ui.List(entities):area(job.area),
		table.unpack(Marker:new(job.area, folder):redraw()),
	})
end

function M:seek(job)
	local folder = cx.active.preview.folder
	if folder and folder.cwd == job.file.url then
		local step = math.floor(job.units * job.area.h / 10)
		local bound = math.max(0, #folder.files - job.area.h)
		ya.mgr_emit("peek", {
			ya.clamp(0, cx.active.preview.skip + step, bound),
			only_if = job.file.url,
		})
	end
end

function M:spot(job) require("file"):spot(job) end

return M
