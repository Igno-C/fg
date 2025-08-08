extends ServerConnector

signal retrieved(pid: int, data: PackedByteArray)
signal request_save(pid: int)

func _ready() -> void:
	var config := ConfigFile.new()
	config.load("res://serverconfig.cfg")
	var ip: String = config.get_value("DbServer", "address")
	var port: int = config.get_value("DbServer", "port")
	var auth_token: String = config.get_value("DbServer", "auth_token")
	
	set_name("db server")
	set_target_name("db")
	set_token(auth_token)
	set_client(port, ip)
	
	start_server()

func save_data(pid: int, data: PackedByteArray) -> void:
	rpc_id(1, "save", pid, data)

func retrieve_data(pid: int, force_create: bool) -> void:
	rpc_id(1, "retrieve", pid, force_create)

@rpc("any_peer", "call_remote", "reliable", 0)
func save(pid: int):
	request_save.emit(pid)

@rpc("any_peer", "call_remote", "reliable", 0)
func retrieve(pid: int, data: PackedByteArray):
	print("Retrieved data from db for pid ", pid, ": ", data)
	retrieved.emit(pid, data)
