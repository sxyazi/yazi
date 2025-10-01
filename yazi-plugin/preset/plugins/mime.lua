local function fetch(_, job)
	ya.notify {
		title = "Deprecated plugin",
		content = "The `mime` fetcher is deprecated, use `mime.file` instead in your `yazi.toml`\n\nSee https://github.com/sxyazi/yazi/pull/3222 for more details.",
		timeout = 15,
		level = "warn",
	}

	return require("mime.file"):fetch(job)
end

return { fetch = fetch }
