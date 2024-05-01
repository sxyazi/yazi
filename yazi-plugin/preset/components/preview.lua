---@class yazi.Preview
Preview = {
	area = ui.Rect.default,
}

---@param area unknown
---@return table
function Preview:render(area)
	self.area = area
	return {}
end
