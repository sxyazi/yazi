local Noop = {}

function Noop:peek() end

function Noop:seek() end

function Noop:preload() return 1 end

return Noop
