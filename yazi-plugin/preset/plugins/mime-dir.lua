local function fetch(_, job)
	local updates = {}
	for _, file in ipairs(job.files) do
		if file.url.scheme.is_virtual then
			updates[file.url] = "folder/remote"
		else
			updates[file.url] = "folder/local"
		end
	end

	ya.emit("update_mimes", { updates = updates })
	return true
end

return { fetch = fetch }
