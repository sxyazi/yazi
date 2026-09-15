local M = {}

function M:provide(job)
	if job.op == "Capabilities" then
		return { reroute = 3 }
	elseif job.op == "Reroute" then
		local target = fs.clean_url(Url(tostring(job.url.path)))
		local file, err = fs.reroute(target)
		if err and err.kind == "Unsupported" then
			return fs.file(target)
		else
			return file, err
		end
	elseif job.op == "Revalidate" then
		return nil
	elseif job.op == "ReadDir" then
		return ya.co(function()
			while true do
				ya.sleep(1000)
			end
		end)
	end

	return false, Err("go:// does not support %s", job.op)
end

return M
