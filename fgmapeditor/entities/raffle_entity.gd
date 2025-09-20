class_name RaffleEntity

extends GenericScriptedEntity

@export var cooldown: float = 45.
@export var reward: int = 100

# net_id -> username
var entered_players: Dictionary[int, String] = {}

var time = 0.

func _ready() -> void:
	interactable = true
	related_scene = "raffle"

func _process(delta: float) -> void:
	if not entered_players.is_empty():
		time += delta
		if time > cooldown:
			time = 0.
			var winner_id: int = entered_players.keys().pick_random()
			var winner_name: String = entered_players[winner_id]
			for net_id in entered_players:
				if net_id == winner_id:
					emit_response(ScriptResponse.chat_message("You won the raffle!", winner_id))
					emit_response(ScriptResponse.change_gold(reward, winner_id))
				else:
					emit_response(
						ScriptResponse.chat_message("Player %s won the raffle!" % winner_name, net_id)
					)
			entered_players.clear()

func _on_player_interaction(player: PlayerContainer, net_id: int) -> Array[ScriptResponse]:
	if entered_players.has(net_id):
		var timeleft := roundi(cooldown - time)
		var message: String = "You are already entered in the raffle. %s seconds left until a winner is picked." % timeleft
		return [ScriptResponse.chat_message(message, net_id)]
	else:
		entered_players[net_id] = player.get_name()
		return [ScriptResponse.chat_message("Entered the raffle.", net_id)]
