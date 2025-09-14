class_name DebugNpc

extends GenericScriptedEntity


func _ready() -> void:
	interactable = true
	related_scene = "debug_npc"

func _process(delta: float) -> void:
	if randi_range(0, 29) == 0:
		var xdelta := randi_range(-1, 1)
		var ydelta := randi_range(-1, 1)
		var newx = pos.x + xdelta
		var newy = pos.y + ydelta
		emit_response(ScriptResponse.move_self(newx, newy, 2))

func _on_player_interaction(player: PlayerContainer, net_id: int) -> Array[ScriptResponse]:
	return [ScriptResponse.chat_message("Hey!", net_id)]
