local function fetch(_, job)
	local updates = {}
	for _, file in ipairs(job.files) do
		updates[file.url] = "folder/local"
	end

	ya.emit("update_mimes", { updates = updates })
	return true
end

return { fetch = fetch }
