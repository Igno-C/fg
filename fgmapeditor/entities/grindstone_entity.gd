class_name GrindstoneEntity

extends GenericScriptedEntity

func _ready() -> void:
	related_scene = "grindstone"
	interactable = true

func _on_player_interaction(player: PlayerContainer, net_id: int) -> Array[ScriptResponse]:
	var equipped_item: ItemResource = player.get_equipped_item()
	if equipped_item == null:
		return [ScriptResponse.chat_message("You have no equipped tool.", net_id)]
	
	var tool_type = equipped_item.custom_data.get("tool")
	if tool_type == null:
		return [ScriptResponse.chat_message("Your equipped item is not a tool.", net_id)]
	
	var power = equipped_item.custom_data.get("power")
	var sharpened = equipped_item.custom_data.get("sharpened")
	if power == null:
		return [] # This one shouldn't happen assuming all tools have a power field
	if sharpened != null:
		return [ScriptResponse.chat_message("Your equipped tool is already sharpened.", net_id)]
	
	equipped_item.custom_data.set("sharpened", 1)
	equipped_item.custom_data.set("power", power + 1)
	equipped_item.name += " [+1]"
	
	return [
		ScriptResponse.take_item(equipped_item.id_string, equipped_item.count, net_id),
		ScriptResponse.give_item(equipped_item, net_id)
	]
