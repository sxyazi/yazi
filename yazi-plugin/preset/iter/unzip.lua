local self <close>, n = ...
assert(math.type(n) == "integer" and n > 0, "expected a positive integer")

local columns = {}
for i = 1, n do
	columns[i] = { n = 0 }
end

local _, err = self:each(function(...)
	for i = 1, n do
		local column = columns[i]
		column.n = column.n + 1
		column[column.n] = select(i, ...)
	end
end)
if err then
	return nil, err
end

return table.unpack(columns, 1, n)
