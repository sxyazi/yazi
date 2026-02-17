local M = {}

function M:setup()
	ps.sub_remote("dds-emit", function(action)
		local i, args = 1, {}
		for j = 2, #action do
			local word = string.char(table.unpack(action[j]))
			local key = word:match("^%-%-([^=]+)")
			if not key then
				i, args[i] = i + 1, word
			elseif #key + 2 == #word then
				args[key] = true
			else
				args[key] = word:sub(#key + 4)
			end
		end
		ya.emit(action[1], args)
	end)
end

return M
