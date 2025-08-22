class_name GenericPortalEntity
extends GenericScriptedEntity

@export var mapname: String
@export var to_where: Vector2i
@export var speed: int

func _on_player_walk(net_id: int) -> ScriptResponse:
	return ScriptResponse.move_player(to_where.x, to_where.y, speed, net_id)
