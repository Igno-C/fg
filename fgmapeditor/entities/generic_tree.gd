class_name GenericTree

extends GenericScriptedEntity

const cooldown: float = 20.
var oak_wood: ItemResource = load("res://items/oak_wood.tres")
var fir_wood: ItemResource = load("res://items/fir_wood.tres")


@export_enum("oak", "fir") var kind: String = "oak"
var time: float = 0.
var cut_down: bool = false

func _ready() -> void:
	interactable = true
	related_scene = "tree"
	public_data["kind"] = kind
	public_data["cut"] = false

func _process(delta: float) -> void:
	if cut_down:
		time += delta
		if time > cooldown:
			time = 0.
			cut_down = false
			set_public_value("cut", false)

func _on_player_interaction(player: PlayerContainer, net_id: int) -> Array[ScriptResponse]:
	if cut_down:
		return [ScriptResponse.null_response()]
	var eq_item := player.get_equipped_item()
	if eq_item == null or eq_item.custom_data["tool"] != "axe":
		return [ScriptResponse.chat_message("You need to equip an axe to cut down a tree.", net_id)]
	#var woodcutting := player.get_stat("woodcutting")
	var axe_power: int = eq_item.custom_data["power"]
	var required_power: int
	var wood_resource: ItemResource
	if kind == "oak":
		required_power = 1
		wood_resource = oak_wood
	elif kind == "fir":
		required_power = 2
		wood_resource = fir_wood
	
	if axe_power >= required_power:
		cut_down = true
		set_public_value("cut", true)
		
		return [
			ScriptResponse.give_item(wood_resource, net_id),
			ScriptResponse.give_xp("woodcutting", required_power * 10, net_id)
		]
	return [ScriptResponse.chat_message("Your axe is too weak to cut down this tree.", net_id)]
