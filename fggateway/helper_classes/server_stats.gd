class_name ServerStats

const MAX_LOAD_PERCENT: float = 0.8
var name := "Server still loading..."
var address := ""
var port := -1

var current_players := 0
var max_players := 0


func _to_string() -> String:
	var s := "Server \"%s\" at %s:%s, players: %s/%s"
	return s % [name, address, port, current_players, max_players]

func max_load() -> bool:
	var percent: float = (current_players as float) / (max_players as float)
	return percent > MAX_LOAD_PERCENT

func to_dict() -> Dictionary:
	var percent: float = (current_players as float) / (max_players as float)
	var load: int
	if max_load():
		load = 4
	elif percent > 0.5:
		load = 3
	elif percent > 0.3:
		load = 2
	else:
		load = 1
	return {
		"name": name,
		"load": load
	}
