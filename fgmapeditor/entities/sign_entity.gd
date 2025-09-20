class_name SignEntity

extends GenericScriptedEntity

@export var text: String = ""

func _ready() -> void:
	interactable = true
	interactable_distance = 3
	related_scene = "sign"

func _on_player_interaction(player: PlayerContainer, net_id: int) -> Array[ScriptResponse]:
	return [ScriptResponse.chat_message(text, net_id)]
