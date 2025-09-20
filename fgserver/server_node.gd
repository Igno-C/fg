extends Server

func _init() -> void:
	var config := ConfigFile.new()
	config.load("res://serverconfig.cfg")
	var max_players: int = config.get_value("Server", "max_players")
	var port: int = config.get_value("Server", "port")
	set_config(port, max_players)
