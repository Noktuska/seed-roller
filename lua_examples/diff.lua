function eval(new_spoiler, old_spoiler)
	if old_spoiler == nil then return true end
	
	get_diff = function(spoiler)
		diff = 0
		for _, details in ipairs(spoiler.details) do
			for _, item in ipairs(details.items) do
				if item.difficulty ~= nil then
					for _, route_entry in ipairs(item.obtain_route) do
						diff = diff + 2 ^ route_entry.strat_difficulty
					end
					break
				end
			end
		end
		return diff
	end
	
	diff_old = get_diff(old_spoiler)
	diff_new = get_diff(new_spoiler)

	print("diff_new: ", diff_new)
	
	return diff_new > diff_old
end

return eval