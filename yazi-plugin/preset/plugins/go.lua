local M = {}

function M:provide(job)
	if job.op == "Capabilities" then
		return { reroute = 3 }
	elseif job.op == "Reroute" then
		local target = fs.clean_url(Url(tostring(job.url.path)))
		local file = fs.reroute(target) or fs.file(target)
		return file or File {
			url = target,
			cha = Cha { kind = 8, mode = tonumber("100600", 8) },
		}
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
