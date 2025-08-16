class_name ServerStats

var name := "ERR"
var address := ""
var port := -1

var current_players := 0
var max_players := 0


func _to_string() -> String:
	var s := "Server \"%s\" at %s:%s, players: %s/%s"
	return s % [name, address, port, current_players, max_players]

func to_dict() -> Dictionary:
	var percent: float = (current_players as float) / (max_players as float)
	var load: int
	if percent > 0.8:
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
