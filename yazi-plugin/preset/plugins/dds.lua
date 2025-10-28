local M = {}

function M:setup()
	ps.sub_remote("dds-emit", function(cmd)
		local i, args = 1, {}
		for j = 2, #cmd do
			local word = string.char(table.unpack(cmd[j]))
			local key = word:match("^%-%-([^=]+)")
			if not key then
				i, args[i] = i + 1, word
			elseif #key + 2 == #word then
				args[key] = true
			else
				args[key] = word:sub(#key + 4)
			end
		end
		ya.emit(cmd[1], args)
	end)
end

return M
