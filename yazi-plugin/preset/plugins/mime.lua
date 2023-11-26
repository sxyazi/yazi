local Mime = {}

function Mime:preload()
	local command = Command.new("file"):arg("--mime-type"):stdout(Command.PIPED):stderr(Command.PIPED)
	if ya.target_family() == "windows" then
		command:arg("-b")
	else
		command:arg("-bL")
	end

	local urls = {}
	for _, file in ipairs(self.files) do
		urls[#urls + 1] = tostring(file.url)
	end

	local i, data = 1, {}
	local output = command:args(urls):output()
	for line in output.stdout:gmatch("[^\r\n]+") do
		if i > #urls then
			break
		end
		if ya.mime_valid(line) then
			data[urls[i]] = line
		end
		i = i + 1
	end

	if #data then
		ya.manager_emit("update_mimetype", {}, data)
		return 3
	end
	return 2
end

return Mime
