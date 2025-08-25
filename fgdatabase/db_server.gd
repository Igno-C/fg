extends ServerConnector

const db_path: String = "res://data/gamedb.db"
var db := SQLite.new()

const data_query: String = "SELECT data, lock_id FROM pdata WHERE pid = ?"
const check_lock_query: String = "SELECT lock_id FROM pdata WHERE pid = ?"
const save_query: String = "UPDATE pdata SET data = ? WHERE pid = ?"
const lock_query: String = "UPDATE pdata SET lock_id = ? WHERE pid = ?"
const unlock_all_query: String = "UPDATE pdata SET lock_id = null WHERE lock_id = ?"
const unlock_absolutely_all_query: String = "UPDATE pdata SET lock_id = null"

#var servers: Dictionary

func _ready() -> void:
	var config := ConfigFile.new()
	config.load("res://db.cfg")
	var port = config.get_value("DbServer", "port")
	var max_servers = config.get_value("DbServer", "max_servers")
	var auth_token = config.get_value("DbServer", "auth_token")
	
	set_name("db-game server")
	set_target_name("game server")
	set_token(auth_token)
	set_server(port, max_servers)
	
	start_server()
	
	multiplayer.peer_disconnected.connect(_on_server_disconnect)
	
	db.verbosity_level = SQLite.VerbosityLevel.NORMAL
	db.path = db_path
	create_or_open_db()
	db.query(unlock_absolutely_all_query)
	
	var gateway_db_server: Node = get_node("/root/GatewayDbServer")
	gateway_db_server.create_new_player.connect(create_new_player)

func create_or_open_db() -> void:
	var exists = FileAccess.file_exists(db_path)
	if not db.open_db():
		print("Failed to open or create db somehow: ", db.error_message)
	
	# Initialize database here
	if not exists:
		print("Database didn't exist, initializing")
		var game_db_dict := {
			"pid": {"data_type":"int", "primary_key": true, "not_null": true, "unique": true},
			"lock_id": {"data_type":"int"},
			"data": {"data_type":"blob", "not_null": true},
		}
		
		if not db.create_table("pdata", game_db_dict):
			print("Failed to create db table somehow: ", db.error_message)

func create_new_player(pid: int, username: String) -> void:
	db.insert_row("pdata", {
		"pid": pid,
		"lock_id": null,
		"data": PlayerContainer.from_name(username, pid).to_bytearray()
	})

# Sets all lock_ids equal net_id to null 
func unlock_all(net_id: int) -> void:
	db.query_with_bindings(unlock_all_query, [net_id])

# Sets lock_id to null for pid
func unlock_pid(pid: int) -> void:
	db.query_with_bindings(lock_query, [null, pid])

# Sets lock_id to net_id for pid
func lock_pid(pid: int, net_id: int) -> void:
	db.query_with_bindings(lock_query, [net_id, pid])

func _on_server_disconnect(net_id: int) -> void:
	unlock_all(net_id)

@rpc("any_peer", "call_remote", "reliable", 0)
func _save(pid: int, data: PackedByteArray, unlock: bool = false) -> void:
	var server_id := multiplayer.get_remote_sender_id()
	db.query_with_bindings(check_lock_query, [pid])
	
	if db.query_result_by_reference.is_empty():
		print("Tried to save to nonexistend pid: ", pid)
		return
	
	var lock_id = db.query_result_by_reference[0]["lock_id"]
	if lock_id != server_id:
		print("Server %s tried to save to entry with pid %s it didn't have lock to" % [server_id, pid])
		print("Lock: ", lock_id)
		return
	
	if unlock:
		unlock_pid(pid)
	
	print("Saving data for pid ", pid)
	db.query_with_bindings(save_query, [data, pid])

@rpc("any_peer", "call_remote", "reliable", 0)
func _retrieve(pid: int, lock: bool = false) -> void:
	var server_id := multiplayer.get_remote_sender_id()
	db.query_with_bindings(data_query, [pid])
	
	if db.query_result_by_reference.is_empty():
		rpc_id(server_id, "_retrieve", pid, PackedByteArray())
		return
	var entry: Dictionary = db.query_result_by_reference[0]
	
	if lock:
		var lock_id = entry["lock_id"]
		if lock_id == null:
			print("Locking entry for pid %s by server %s" % [pid, server_id])
			lock_pid(pid, server_id)
		else:
			print("Server %s attempted lock on already locked entry for pid %s" % [server_id, pid])
			rpc_id(server_id, "_retrieve", pid, PackedByteArray())
			return
	
	var data: PackedByteArray = entry["data"]
	print("Retrieved data for pid ", pid)
	
	rpc_id(server_id, "_retrieve", pid, data)

@rpc("any_peer", "call_remote", "reliable", 1)
func relay_dm(from: String, text: String, target_pid: int) -> void:
	db.query_with_bindings(check_lock_query, [target_pid])
	
	if db.query_result_by_reference.is_empty():
		return
	
	var lock_id = db.query_result_by_reference[0]["lock_id"]
	if lock_id != null:
		rpc_id(lock_id, "relay_dm", from, text, target_pid)
	else:
		print("Received dm relay for unlocked pid: ", target_pid)
