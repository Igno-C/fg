class_name GenericCooldownChest

extends GenericScriptedEntity

@export var cooldown: float = 60.
@export var loot: ItemResource

var time = 0.
var loot_ready: bool = true

func _ready() -> void:
	interactable = true
	related_scene = "chest"
	public_data["open"] = false

func _process(delta: float) -> void:
	if not loot_ready:
		time += delta
		if time > cooldown:
			time = 0.
			loot_ready = true
			set_public_value("open", false)

func _on_player_interaction(player: PlayerContainer, net_id: int) -> Array[ScriptResponse]:
	if loot_ready:
		loot_ready = false
		set_public_value("open", true)
		print("giving loot of ", loot, " to ", net_id)
		return [ScriptResponse.give_item(loot, net_id)]
	else:
		return [ScriptResponse.null_response()]
