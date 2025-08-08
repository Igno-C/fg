extends ServerConnector

#var network := ENetMultiplayerPeer.new()
#var ip: String
#var port: int
#var auth_token: String

var config := ConfigFile.new()

signal response(net_id: int, valid: bool)
signal created_response(net_id: int, valid: bool)

func _ready() -> void:
	config.load("res://gateway.cfg")
	var address: String = config.get_value("Auth", "address")
	var port: int = config.get_value("Auth", "port")
	var auth_token: String = config.get_value("Auth", "auth_token")
	
	set_name("auth")
	set_target_name("gateway")
	set_token(auth_token)
	set_client(port, address)
	
	start_server()

func validate(net_id: int, username: String, password: String):
	var err = rpc_id(1, "authenticate", net_id, username, password)
	if err != OK:
		printerr("Error on rpc for ", net_id, " in auth client: ", err)

func create_new_account(net_id: int, email: String, username: String, password: String) -> void:
	var err = rpc_id(1, "create_account", net_id, email, username, password)
	if err != OK:
		printerr("Error on rpc for ", net_id, " in auth client: ", err)

@rpc("any_peer", "call_remote", "reliable", 0)
func authenticate(net_id: int, pid: int) -> void:
	response.emit(net_id, pid)

@rpc("any_peer", "call_remote", "reliable", 0)
func create_account(net_id: int, new_pid: int) -> void:
	print("Created account for ", net_id, " with pid ", new_pid)
	if new_pid == -1:
		created_response.emit(net_id, false)
	else:
		created_response.emit(net_id, true)
