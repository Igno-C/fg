extends ServerConnector

func _ready() -> void:
	var config := ConfigFile.new()
	config.load("res://gateway.cfg")
	var address: String = config.get_value("GatewayDb", "address")
	var port = config.get_value("GatewayDbr", "port")
	var auth_token = config.get_value("GatewayDb", "auth_token")
	
	set_name("gateway-db")
	set_target_name("db")
	set_token(auth_token)
	set_auto_reconnect(true)
	set_client(port, address)
	
	start_server()

func create_new_player(pid: int, username: String) -> void:
	rpc_id(1, "_create_new_player", pid, username)

@rpc("any_peer", "call_remote", "reliable", 0)
func _create_new_player() -> void:
	#create_new_player.emit(pid, username)
	pass
