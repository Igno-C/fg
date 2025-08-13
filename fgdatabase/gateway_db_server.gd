extends ServerConnector

signal create_new_player(pid: int, username: String)

func _ready() -> void:
	var config := ConfigFile.new()
	config.load("res://db.cfg")
	var port = config.get_value("GatewayDb", "port")
	var max_gateways = config.get_value("GatewayDb", "max_gateways")
	var auth_token = config.get_value("GatewayDb", "auth_token")
	
	set_name("db-gateway")
	set_target_name("gateway")
	set_token(auth_token)
	set_server(port, max_gateways)
	
	start_server()

@rpc("any_peer", "call_remote", "reliable", 0)
func _create_new_player(pid: int, username: String) -> void:
	create_new_player.emit(pid, username)
