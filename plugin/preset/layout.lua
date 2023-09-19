local Layout = {}

function Layout:new(...)
	local o = {
		direction = "R",
		dimension = {},
		elements = { ... },
	}
	setmetatable(o, self)
	self.__index = self
	return o
end

function Layout:from(elements) return self:new(table.unpack(elements)) end

function Layout:rows(rows)
	self.direction = "R"
	self.dimension = rows
	return self:to_string()
end

function Layout:cols(cols)
	self.direction = "C"
	self.dimension = cols
	return self:to_string()
end

function Layout:to_string()
	local s = ""
	for i, element in ipairs(self.elements) do
		if i == 1 then
			s = s .. self.direction
		end
		s = s .. self.dimension[i] .. "\0" .. element:to_string():gsub("\0", "\\\0")
	end
end

setmetatable(Layout, {
	__call = function(self, ...) return self:new(...) end,
})

yazi = yazi or {}
yazi.Layout = Layout
