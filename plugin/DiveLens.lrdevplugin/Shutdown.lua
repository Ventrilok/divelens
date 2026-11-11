--[[
Runs when Lightroom quits (LrShutdownApp: uses the returned function, which
waits for the link to close) or unloads the plugin (LrShutdownPlugin: the
script is only run). Either way the link says goodbye to the app, closes and
removes the link file.

Part of DiveLens, GPL-3.0-or-later.
]]

local LrTasks = import "LrTasks"

require "Link".stop()

return {
    LrShutdownFunction = function(doneFunction, progressFunction)
        LrTasks.startAsyncTask(function()
            require "Link".stop()
            local waited = 0
            while not require "Link".isStopped() and waited < 2 do
                progressFunction(waited / 2)
                LrTasks.sleep(0.1)
                waited = waited + 0.1
            end
            progressFunction(1)
            doneFunction()
        end)
    end,
}
