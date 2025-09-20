extends ServerConnector

const db_path: String = "res://data/authdb.db"
var db := SQLite.new()

const password_query: String = "SELECT password, salt, pid FROM auth WHERE username = ?"

const index_query: String = "CREATE INDEX idx_username ON auth(username)"

func _ready() -> void:
	var prev_ticks := Time.get_ticks_usec()
	hash_password("ExamplePassword12345", get_salt())
	var elapsed = Time.get_ticks_usec() - prev_ticks
	print("Hashing password takes %s µs" % elapsed)
	
	var config := ConfigFile.new()
	config.load("res://auth.cfg")
	var port = config.get_value("Auth", "port")
	var max_gateways = config.get_value("Auth", "max_gateways")
	var auth_token = config.get_value("Auth", "auth_token")
	
	set_name("auth")
	set_target_name("gateway")
	set_token(auth_token)
	
	var crypto_key := CryptoKey.new()
	crypto_key.load("res://certificates/X509_Key.key")
	var certificate := X509Certificate.new()
	certificate.load("res://certificates/X509_Certificate.crt")
	
	set_server_dtls(port, max_gateways, crypto_key, certificate)
	
	start_server()
	
	db.verbosity_level = SQLite.VerbosityLevel.NORMAL
	db.path = db_path
	create_or_open_db()

func create_or_open_db() -> void:
	var exists = FileAccess.file_exists(db_path)
	if not db.open_db():
		print("Failed to open or create db somehow: ", db.error_message)
	
	# Initialize database here
	if not exists:
		print("Database didn't exist, initializing")
		var auth_db_dict := {
			"pid": {"data_type":"int", "primary_key": true, "not_null": true, "auto_increment": true, "unique": true},
			#"email": {"data_type":"char(40)", "not_null": true, "unique": true},
			"username": {"data_type":"char(20)", "not_null": true, "unique": true},
			"password": {"data_type":"char(40)", "not_null": true},
			"salt": {"data_type":"text", "not_null": true},
		}
		
		if not db.create_table("auth", auth_db_dict):
			print("Failed to create db table somehow: ", db.error_message)
		if not db.query(index_query):
			print("Failed to create username index: ", db.error_message)

func check_with_db(username: String, password: String) -> int:
	if not db.query_with_bindings(password_query, [username]):
		printerr("error querying database: ", db.error_message)
		return -1
	if db.query_result_by_reference.is_empty():
		return -1
	
	var result: Dictionary = db.query_result_by_reference[0]
	var hashed_password: String = result["password"]
	var salt: String = result["salt"]
	var newhash := hash_password(password, salt)
	return result["pid"]

func insert_salted(username: String, password: String) -> int:
	username = username.to_lower()
	#email = email.to_lower()
	
	var salt := get_salt()
	var hashed_password := hash_password(password, salt)
	
	if not db.insert_row("auth", {
		#"email": email,
		"username": username,
		"password": hashed_password,
		"salt": salt
	}):
		return -1
	
	var pid := db.last_insert_rowid
	print("Created account for ", username, " with pid ", pid)
	
	return pid

func get_salt() -> String:
	randomize()
	return str(randi()).sha256_text()

func hash_password(password: String, salt: String) -> String:
	var hashed_password := password
	
	var rounds := 2**18
	while rounds > 0:
		hashed_password = (hashed_password + salt).sha256_text()
		rounds -= 1
	
	return hashed_password

@rpc("any_peer", "call_remote", "reliable", 0)
func _authenticate(net_id: int, username: String, password: String):
	var gateway_id := multiplayer.get_remote_sender_id()
	
	var pid := check_with_db(username, password)
	rpc_id(gateway_id, "_authenticate", net_id, pid)

@rpc("any_peer", "call_remote", "reliable", 0)
func _create_account(net_id: int, username: String, password: String) -> void:
	var gateway_id := multiplayer.get_remote_sender_id()
	
	var new_pid: int = -1
	if username.length() <= 20 and username.length() >= 5:
		if password.length() <= 40 or password.length() >= 8:
			new_pid = insert_salted(username, password)
	
	rpc_id(gateway_id, "_create_account", net_id, new_pid)
