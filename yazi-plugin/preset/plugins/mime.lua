local M = {}

function M:preload()
	local command = Command("file"):arg("--mime-type"):stdout(Command.PIPED):stderr(Command.PIPED)
	if ya.target_family() == "windows" then
		command:arg("-b")
	else
		command:arg("-bL")
	end

	local urls = {}
	for _, file in ipairs(self.files) do
		urls[#urls + 1] = tostring(file.url)
	end

	local i, mimes = 1, {}
	local output = command:args(urls):output()
	for line in output.stdout:gmatch("[^\r\n]+") do
		if i > #urls then
			break
		end
		if ya.mime_valid(line) then
			mimes[urls[i]] = line
		end
		i = i + 1
	end

	if #mimes then
		ya.manager_emit("update_mimetype", {}, mimes)
		return 3
	end
	return 2
end

return M
