local M = {}

function M:peek(job) ya.preview_widget(job, {}) end

function M:seek() end

function M:fetch(job)
	return ya.co(function()
		for _, file in ipairs(job.files) do
			coroutine.yield(file)
		end
	end)
end

function M:preload() return true end

function M:spot() end

return M
