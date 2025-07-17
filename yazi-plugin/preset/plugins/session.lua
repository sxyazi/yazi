local function setup(_, opts)
	if opts.sync_yanked then
		ps.sub_remote("@yank", function(opt) ya.emit("update_yanked", { opt = opt }) end)
	end
end

return { setup = setup }
