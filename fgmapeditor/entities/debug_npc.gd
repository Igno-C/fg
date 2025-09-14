class_name DebugNpc

extends GenericScriptedEntity

const greetings: Array[String] = [
	"Hey!",
	"Howdy!",
	"What's up!"
]
@export var max_home_distance: int = 2
var home: Vector2i
var ticks_till_next_move = 20

func _ready() -> void:
	home = pos
	interactable = true
	related_scene = "debug_npc"

func _process(delta: float) -> void:
	ticks_till_next_move -= 1
	if ticks_till_next_move <= 0:
		ticks_till_next_move = randi_range(15, 60)
		var xdelta := randi_range(-1, 1)
		var ydelta := randi_range(-1, 1)
		var newx = pos.x + xdelta
		var newy = pos.y + ydelta
		if abs(newx - home.x) > max_home_distance:
			newx = pos.x
		if abs(newy - home.y) > max_home_distance:
			newy = pos.y
		emit_response(ScriptResponse.move_self(newx, newy, 2))

func _on_player_interaction(player: PlayerContainer, net_id: int) -> Array[ScriptResponse]:
	return [ScriptResponse.chat_message(greetings.pick_random(), net_id)]
