Preview = {
	area = ui.Rect.default,
}

function Preview:render(area)
	self.area = area
	return {}
end

function Preview:click(event, up)
	if up or not event.is_left then
		return
	end

	local window = Folder:window(Folder.PREVIEW) or {}
	if window[event.y] then
		ya.manager_emit("reveal", { window[event.y].url })
	else
		ya.manager_emit("enter", {})
	end
end

function Preview:scroll(event, step) ya.manager_emit("seek", { step }) end

function Preview:touch(event, step) end
