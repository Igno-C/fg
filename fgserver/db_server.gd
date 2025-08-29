extends ServerConnector

signal retrieved(pid: int, data: PackedByteArray)
signal request_save(pid: int)
signal dm_received(from: String, text: String, target_pid: int)

func _ready() -> void:
	var config := ConfigFile.new()
	config.load("res://serverconfig.cfg")
	var ip: String = config.get_value("DbServer", "address")
	var port: int = config.get_value("DbServer", "port")
	var auth_token: String = config.get_value("DbServer", "auth_token")
	
	set_name("db server")
	set_target_name("db")
	set_token(auth_token)
	set_auto_reconnect(true)
	set_client(port, ip)
	
	start_server()

func save(pid: int, data: PackedByteArray, unlock: bool) -> void:
	print("Saving data for pid ", pid);
	rpc_id(1, "_save", pid, data, unlock)

func retrieve(pid: int, lock: bool) -> void:
	rpc_id(1, "_retrieve", pid, lock)

@rpc("any_peer", "call_remote", "reliable", 0)
func _save(pid: int) -> void:
	pass
	#request_save.emit(pid)

@rpc("any_peer", "call_remote", "reliable", 0)
func _retrieve(pid: int, data: PackedByteArray) -> void:
	print("Retrieved data from db for pid ", pid)
	retrieved.emit(pid, data)

@rpc("any_peer", "call_remote", "reliable", 1)
func relay_dm(from: String, text: String, target_pid: int) -> void:
	dm_received.emit(from, text, target_pid)
