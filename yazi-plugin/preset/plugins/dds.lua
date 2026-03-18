local M = {}

function M.parse_args(t, i)
	local j, args = 1, {}
	for i = i, #t do
		local word = string.char(table.unpack(t[i]))
		local key = word:match("^%-%-([^=]+)")
		if not key then
			j, args[j] = j + 1, word
		elseif #key + 2 == #word then
			args[key] = true
		else
			args[key] = word:sub(#key + 4)
		end
	end
	return args
end

function M:setup()
	ps.sub_remote("dds-emit", function(t) ya.emit(t[1], M.parse_args(t, 2)) end)

	ps.sub_remote("dds-exec", function(t)
		ya.async(function()
			local ok, value = pcall(ya.exec, t[2], M.parse_args(t, 3))
			ps.pub_to(t[1], "dds-exec-result", ok and {
				ok = true,
				value = value,
			} or {
				ok = false,
				error = tostring(value),
			})
		end)
	end)
end

return M
