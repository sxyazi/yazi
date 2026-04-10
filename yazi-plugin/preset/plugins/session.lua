local function setup(_, opts)
	if opts.sync_yanked then
		ps.sub_remote("@yank", function(state) ya.emit("update_yanked", { state }) end)
	end
end

return { setup = setup }
