-- TODO: remove this after 0.3.0 release

Manager = {}
Folder = {}
File = {}

local b = false
function __yazi_check_and_warn_deprecated_api()
	if b then
		return
	end

	local warn = function(name)
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

	b = true
	for _ in pairs(Manager) do
		warn("Manager")
		break
	end
	for _ in pairs(Folder) do
		warn("Folder")
		break
	end
	for _ in pairs(File) do
		warn("File")
		break
	end
end
