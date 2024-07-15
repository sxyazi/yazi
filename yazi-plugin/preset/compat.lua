-- TODO: remove this after 0.3.0 release

Manager = {}
Folder = {}
File = {}

local function warn(name)
	ya.notify {
		title = "Deprecated API",
		content = string.format(
			[[The `%s` global variable has been removed in Yazi v0.3, please remove it from your `init.lua`.

See https://github.com/sxyazi/yazi/pull/1257 for details.]],
			name
		),
		timeout = 20,
		level = "warn",
	}
end

local b1, b2, b3 = false, false, false
function __yazi_check_and_warn_deprecated_api()
	if not b1 then
		for _ in pairs(Manager) do
			b1 = true
			warn("Manager")
			break
		end
	end

	if not b2 then
		for _ in pairs(Folder) do
			b2 = true
			warn("Folder")
			break
		end
	end

	if not b3 then
		for _ in pairs(File) do
			b3 = true
			warn("File")
			break
		end
	end
end
