---
--- Created by recore
--- DateTime: 8/11/26 8:50 pm
---

function mergeTable(...)
    local result = {}
    local index = 1

    for arg = 1, select("#", ...) do
        local source = select(arg, ...)
        for i = 1, #source do
            local value = rawget(source, i)
            if value ~= nil then
                rawset(result, index, value)
                index = index + 1
            end
        end
    end

    return result
end
