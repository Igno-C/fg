class_name DebugNpc

extends GenericScriptedEntity



func _ready() -> void:
	related_scene = "debug_npc"
	public_data["cut"] = false

func _process(delta: float) -> void:
	if randi_range(0, 19) == 0:
		var xdelta := randi_range(-1, 1)
		var ydelta := randi_range(-1, 1)
		emit_response(ScriptResponse.move_self(pos.x + xdelta, pos.y + ydelta, 2))

func _on_player_interaction(player: PlayerContainer, net_id: int) -> ScriptResponse:
	return ScriptResponse.chat_message("Hey!", net_id)
