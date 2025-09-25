class_name GenericTollTeleport

extends GenericScriptedEntity

@export var mapname: String
@export var to_where: Vector2i
@export var cost: int = 0

func _ready() -> void:
	interactable = true
	related_scene = "toll_portal"

func _on_player_interaction(player: PlayerContainer, net_id: int) -> Array[ScriptResponse]:
	var player_gold = player.get_gold()
	if player_gold >= cost:
		return [
			ScriptResponse.change_gold(-cost, net_id),
			ScriptResponse.move_player_to_map(mapname, to_where.x, to_where.y, net_id)
		]
	else:
		return [ScriptResponse.chat_message("This teleport costs %s gold to use!" % cost, net_id)]
