class_name GenericInstancePortalEntity
extends GenericScriptedEntity

@export var mapname: String
@export var to_where: Vector2i

func _ready() -> void:
	walkable = true

func _on_player_walk(player: PlayerContainer, net_id: int) -> ScriptResponse:
	return ScriptResponse.move_player_to_map(mapname, to_where.x, to_where.y, net_id)
