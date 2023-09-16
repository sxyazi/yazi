local Span = {}

function Span:new(content)
	local o = {
		content = content,
		foreground = "",
		background = "",
		modifier = 0,
	}
	setmetatable(o, self)
	self.__index = self
	return o
end

function Span:fg(color)
	self.foreground = color
	return self
end

function Span:bg(color)
	self.background = color
	return self
end

function Span:bold()
	self.modifier = self.modifier | 1
	return self
end

function Span:dim()
	self.modifier = self.modifier | 2
	return self
end

function Span:italic()
	self.modifier = self.modifier | 4
	return self
end

function Span:underline()
	self.modifier = self.modifier | 8
	return self
end

function Span:blink()
	self.modifier = self.modifier | 16
	return self
end

function Span:blink_rapid()
	self.modifier = self.modifier | 32
	return self
end

function Span:reverse()
	self.modifier = self.modifier | 64
	return self
end

function Span:hidden()
	self.modifier = self.modifier | 128
	return self
end

function Span:crossed()
	self.modifier = self.modifier | 256
	return self
end

function Span:reset()
	self.foreground = ""
	self.background = ""
	self.modifier = 0
	return self
end

function Span:style(style)
	if style.fg then
		self:fg(style.fg)
	end
	if style.bg then
		self:bg(style.bg)
	end
	if style.bold then
		self:bold()
	end
	if style.dim then
		self:dim()
	end
	if style.italic then
		self:italic()
	end
	if style.underline then
		self:underline()
	end
	if style.blink then
		self:blink()
	end
	if style.blink_rapid then
		self:blink_rapid()
	end
	if style.reverse then
		self:reverse()
	end
	if style.hidden then
		self:hidden()
	end
	if style.crossed then
		self:crossed()
	end
	return self
end

function Span:to_string()
	return string.format("%s,%s,,%s;%s", self.foreground, self.background, self.modifier, self.content)
end

setmetatable(Span, {
	__call = function(self, content) return self:new(content) end,
	__tostring = function(self) return self:to_string() end,
})

yazi = yazi or {}
yazi.Span = Span
