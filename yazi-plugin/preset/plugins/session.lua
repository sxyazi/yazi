local function setup(_, opts)
	if opts.sync_yanked then
		ps.sub_remote("@yank", function(body) ya.emit("update_yanked", { cut = body.cut, urls = body }) end)
	end
end

return { setup = setup }
