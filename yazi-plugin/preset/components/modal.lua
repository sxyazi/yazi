Modal = {
	_id = "modal",

	_inc = 1000,
	_children = {},
}

function Modal:new(area) return setmetatable({ _area = area }, { __index = self }) end

function Modal:reflow()
	local components = {}
	for _, child in ipairs(self._children) do
		ya.iter(child[1]:new(self._area):reflow()):into(components)
	end
	return components
end

function Modal:redraw() return {} end

-- Children
function Modal:children_add(tbl, order)
	self._inc = self._inc + 1
	self._children[#self._children + 1] = { tbl, id = self._inc, order = order }

	table.sort(self._children, function(a, b) return a.order < b.order end)
	return self._inc
end

function Modal:children_remove(id)
	for i, child in ipairs(self._children) do
		if child.id == id then
			table.remove(self._children, i)
			break
		end
	end
end

function Modal:children_redraw()
	local elements = {}
	for _, child in ipairs(self._children) do
		ya.iter(ui.redraw(child[1]:new(self._area))):into(elements)
	end
	return elements
end
