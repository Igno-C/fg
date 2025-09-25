class_name FairySpawnerEntity

extends GenericScriptedEntity

var fairy: GenericScriptedEntity = null

var timer: int = 0
var respawn_cooldown: int = 0

func _process(delta: float) -> void:
	if fairy == null:
		timer += 1
		if timer > respawn_cooldown:
			timer = 0
			respawn_cooldown = randi_range(50, 300)
			fairy = FairyEntity.new()
			fairy.pos = pos
			add_child(fairy)
			emit_response(ScriptResponse.register_entity(fairy))
