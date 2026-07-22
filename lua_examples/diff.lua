function eval(new_spoiler, old_spoiler)
	get_diff = function(spoiler)
		diff = 0
		seen_tech = {}
		for _, details in ipairs(spoiler.details) do
			for _, item in ipairs(details.items) do
				if item.difficulty ~= nil then
					for _, route_entry in ipairs(item.obtain_route) do
						if route_entry.strat_id ~= nil then
							if not seen_tech[route_entry.strat_id] then
								diff = diff + 2 ^ route_entry.strat_difficulty
							end
							seen_tech[route_entry.strat_id] = true
						else
							diff = diff + 2 ^ route_entry.strat_difficulty
						end
					end
					break
				end
			end
		end
		return diff
	end
	
	diff_old = 0
	if old_spoiler ~= nil then
		diff_old = get_diff(old_spoiler)
	end
	diff_new = get_diff(new_spoiler)

	print("diff_new: ", diff_new)
	
	return diff_new > diff_old
end

return eval