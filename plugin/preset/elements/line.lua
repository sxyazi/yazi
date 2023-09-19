local Line = {}

function Line:new(...)
	local o = {
		spans = { ... },
	}
	setmetatable(o, self)
	self.__index = self
	return o
end

function Line:from(spans) return self:new(table.unpack(spans)) end

function Line:to_string()
	local s = ""
	for _, span in ipairs(self.spans) do
		s = s .. span:to_string():gsub("\n", "\\\n") .. "\n"
	end
	return s.sub(s, 1, -2)
end

setmetatable(Line, {
	__call = function(self, ...) return self:new(...) end,
	__tostring = function(self) return self:to_string() end,
})

yazi = yazi or {}
yazi.Line = Line
