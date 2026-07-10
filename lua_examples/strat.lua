function eval(new_spoiler, old_spoiler)
	if old_spoiler == nil then return false end
	
	for _, details in ipairs(spoiler.details) do
		for _, item in ipairs(details.items) do
			for _, route_entry in ipairs(item.obtain_route) do
				if route_entry.room_id == 144 and route_entry.strat_id == 93 then return true end
			end
		end
	end
	
	return false
end

return eval