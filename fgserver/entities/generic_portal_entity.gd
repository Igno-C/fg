class_name GenericPortalEntity
extends GenericScriptedEntity

@export var mapname: String
@export var to_where: Vector2i
@export var speed: int

func _ready() -> void:
	walkable = true
	related_scene = "portal"

func _on_player_walk(player: PlayerContainer, net_id: int) -> Array[ScriptResponse]:
	return [ScriptResponse.move_player(to_where.x, to_where.y, speed, net_id)]
