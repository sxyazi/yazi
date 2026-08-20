local M = {}

function M:fetch(job)
	return ya.co(function()
		local mime, updates = nil, {}
		for _, file in ipairs(job.files) do
			if file.url.spec.scheme == "sftp" then
				mime = "folder/remote"
			elseif file.url.spec.scheme == "trash" then
				mime = "trash/folder"
			else
				mime = "folder/local"
			end

			if coroutine.yield(file, { mime }) then
				updates[file.url] = mime
			end
		end

		return M.commit(updates)
	end)
end

function M.commit(updates)
	if next(updates) then
		ya.emit("update_mimes", { updates = updates })
	end
end

return M
