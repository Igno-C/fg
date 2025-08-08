extends ServerConnector

const db_path: String = "res://data/gamedb.db"
var db := SQLite.new()

const data_query: String = "SELECT data FROM pdata WHERE pid = ?"
const save_query: String = "UPDATE pdata SET data = ? WHERE pid = ?"

#var servers: Dictionary

func _ready() -> void:
	var config := ConfigFile.new()
	config.load("res://db.cfg")
	var port = config.get_value("DbServer", "port")
	var max_gateways = config.get_value("DbServer", "max_servers")
	var auth_token = config.get_value("DbServer", "auth_token")
	
	set_name("db")
	set_target_name("game server")
	set_token(auth_token)
	set_server(port, max_gateways)
	
	start_server()
	
	db.verbosity_level = SQLite.VerbosityLevel.NORMAL
	db.path = db_path
	create_or_open_db()
	
	retrieve(1, true)

func create_or_open_db() -> void:
	var exists = FileAccess.file_exists(db_path)
	if not db.open_db():
		print("Failed to open or create db somehow: ", db.error_message)
	
	# Initialize database here
	if not exists:
		print("Database didn't exist, initializing")
		var game_db_dict := {
			"pid": {"data_type":"int", "primary_key": true, "not_null": true, "unique": true},
			"data": {"data_type":"blob", "not_null": true},
		}
		
		if not db.create_table("pdata", game_db_dict):
			print("Failed to create db table somehow: ", db.error_message)

func create_new_playerdata(pid: int) -> void:
	db.insert_row("pdata", {
		"pid": pid,
		"data": PlayerContainer.default().to_bytearray()
	})

@rpc("any_peer", "call_remote", "reliable", 0)
func save(pid: int, data: PackedByteArray) -> void:
	var server_id := multiplayer.get_remote_sender_id()
	if not db.query_with_bindings(save_query, [data, pid]):
		print("Failed to save data for pid ", pid, ": ", db.error_message)

# force_create means that if pid doesn't have data, it should be created
@rpc("any_peer", "call_remote", "reliable", 0)
func retrieve(pid: int, force_create: bool = false) -> void:
	var server_id := multiplayer.get_remote_sender_id()
	db.query_with_bindings(data_query, [pid])
	
	var data: PackedByteArray
	if db.query_result_by_reference.is_empty():
		if force_create:
			create_new_playerdata(pid)
			db.query_with_bindings(data_query, [pid])
			data = db.query_result_by_reference[0]["data"]
	else:
		data = db.query_result_by_reference[0]["data"]
	
	rpc_id(server_id, "retrieve", pid, data)
