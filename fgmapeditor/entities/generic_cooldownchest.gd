class_name GenericCooldownChest

extends GenericScriptedEntity

@export var cooldown: float = 60.
@export var loot: ItemResource = null
@export var gold: int = 0

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
		if loot != null and gold != 0:
			return [ScriptResponse.give_item(loot, net_id), ScriptResponse.change_gold(gold, net_id)]
		elif gold != 0:
			return [ScriptResponse.change_gold(gold, net_id)]
		elif loot != null:
			return [ScriptResponse.give_item(loot, net_id)]
		else:
			return [ScriptResponse.null_response()]
	else:
		return [ScriptResponse.null_response()]
