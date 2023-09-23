local Paragraph = {}
local Alignment = {
	LEFT = 0,
	CENTER = 1,
	RIGHT = 2,
}

function Paragraph:new(...)
	local o = {
		alignment = 0,
		position = nil,
		lines = { ... },
	}
	setmetatable(o, self)
	self.__index = self
	return o
end

function Paragraph:from(lines) return self:new(table.unpack(lines)) end

function Paragraph:align(align)
	self.alignment = align
	return self
end

function Paragraph:area(rect)
	self.position = rect
	return self
end

function Paragraph:to_string()
	local s = ""
	for _, line in ipairs(self.lines) do
		s = s .. line:to_string():gsub("\r", "\\\r") .. "\r"
	end
	return s.sub(s, 1, -2)
end

function Paragraph.render(...)
	local s = "R"
	for _, paragraph in ipairs { ... } do
		s = s
			.. paragraph.position.x
			.. ","
			.. paragraph.position.y
			.. ","
			.. paragraph.position.width
			.. ","
			.. paragraph.position.height
			.. ","
			.. paragraph.alignment
			.. ";"
			.. paragraph:to_string():gsub("\0", "\\\0")
			.. "\0"
	end
	return s.sub(s, 1, -2)
end

setmetatable(Paragraph, {
	__call = function(self, ...) return self:new(...) end,
	__tostring = function(self) return self:to_string() end,
})

ui = ui or {}
ui.Paragraph = Paragraph
ui.Alignment = Alignment
