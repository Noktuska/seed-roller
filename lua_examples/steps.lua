function(new_spoiler, old_spoiler)
	if old_spoiler == nil then return true end
	return #new_spoiler.summary > #old_spoiler.summary
end