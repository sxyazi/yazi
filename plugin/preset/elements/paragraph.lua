local Paragraph = {}

function Paragraph:new(...)
	local o = {
		lines = { ... },
	}
	setmetatable(o, self)
	self.__index = self
	return o
end

function Paragraph:from(lines) return self:new(table.unpack(lines)) end

function Paragraph:to_string()
	local s = ""
	for _, line in ipairs(self.lines) do
		s = s .. line:to_string():gsub("\r", "\\\r") .. "\r"
	end
	return s.sub(s, 1, -2)
end

setmetatable(Paragraph, {
	__call = function(self, ...) return self:new(...) end,
	__tostring = function(self) return self:to_string() end,
})

yazi = yazi or {}
yazi.Paragraph = Paragraph
