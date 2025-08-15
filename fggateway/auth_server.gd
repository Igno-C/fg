extends ServerConnector

#var network := ENetMultiplayerPeer.new()
#var ip: String
#var port: int
#var auth_token: String

var config := ConfigFile.new()

signal response(net_id: int, valid: bool)
signal created_response(net_id: int, pid: int)

func _ready() -> void:
	config.load("res://gateway.cfg")
	var address: String = config.get_value("Auth", "address")
	var port: int = config.get_value("Auth", "port")
	var auth_token: String = config.get_value("Auth", "auth_token")
	
	set_name("gateway auth")
	set_target_name("auth")
	set_token(auth_token)
	set_auto_reconnect(true)
	set_client(port, address)
	
	start_server()

func authenticate(net_id: int, username: String, password: String):
	var err = rpc_id(1, "_authenticate", net_id, username, password)
	if err != OK:
		printerr("Error on rpc for ", net_id, " in auth client: ", err)

func create_account(net_id: int, username: String, password: String) -> void:
	var err = rpc_id(1, "_create_account", net_id, username, password)
	if err != OK:
		printerr("Error on rpc for ", net_id, " in auth client: ", err)

@rpc("any_peer", "call_remote", "reliable", 0)
func _authenticate(net_id: int, pid: int) -> void:
	response.emit(net_id, pid)

@rpc("any_peer", "call_remote", "reliable", 0)
func _create_account(net_id: int, new_pid: int) -> void:
	print("Created account for ", net_id, " with pid ", new_pid)
	if new_pid == -1:
		created_response.emit(net_id, new_pid)
	else:
		created_response.emit(net_id, new_pid)
