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
	for _, el in ipairs(self.spans) do
		local mt = getmetatable(el)
		if mt == ui.Line then
			for _, span in ipairs(el.spans) do
				s = s .. span:to_string():gsub("\n", "\\\n") .. "\n"
			end
		else
			s = s .. el:to_string():gsub("\n", "\\\n") .. "\n"
		end
	end
	return s.sub(s, 1, -2)
end

setmetatable(Line, {
	__call = function(self, ...) return self:new(...) end,
	__tostring = function(self) return self:to_string() end,
})

ui = ui or {}
ui.Line = Line
