extends Node

var network = ENetMultiplayerPeer.new()
var ip: String
var port: int

# Used to temporarily hold the username and password while waiting for gateway connection
var email_held: String = ""
var username_held: String = ""
var password_held: String = ""

var succeeded: bool = false # As in disconnected, but not because of timeout
var waiting: bool = false # Currently awaiting response from gateway
var time: float = 0. # Time spent waiting
const TIMEOUT_THRESHOLD: float = 10.

# Generic connection error signals
signal timeout
signal unreachable
signal other_error(err: int)

# Login signals
signal success
signal invalid
signal joined_server(token: String, ip: String, port: int)

# Account creation signal
signal creation_status(valid: bool)

# Server list signal
signal got_server_list(list: Array[Dictionary])

func _ready() -> void:
	var config := ConfigFile.new()
	config.load("res://default_gateway.cfg")
	ip = config.get_value("Gateway", "ip")
	port = config.get_value("Gateway", "port")
	
	get_tree().set_multiplayer(SceneMultiplayer.new(), get_path())
	multiplayer.peer_connected.connect(_peer_connected)
	multiplayer.peer_disconnected.connect(_peer_disconnected)

# Is run once connected to gateway
# Which rpc is run depends on what data was input
func send_held_data() -> void:
	if email_held.is_empty():
		if username_held.is_empty():
			print("getting server list")
			rpc_id(1, "get_server_list")
		else:
			print("logging in")
			rpc_id(1, "log_in", username_held, password_held)
	else:
		print("creating account")
		rpc_id(1, "create_account", email_held, username_held, password_held)
		email_held = ""
	username_held = ""
	password_held = ""

func _process(delta: float) -> void:
	if waiting:
		time += delta
		if time > TIMEOUT_THRESHOLD:
			unreachable.emit()
			multiplayer.multiplayer_peer.close()
			reset_wait()

func send_credentials(username: String, password: String) -> void:
	username_held = username
	password_held = password
	start_client()

func send_creation(email: String, username: String, password: String) -> void:
	email_held = email
	username_held = username
	password_held = password
	start_client()

func send_server_list_request() -> void:
	rpc_id(1, "get_server_list")
	#start_client()

func send_chosen_server(name: String) -> void:
	rpc_id(1, "join_server", name)

func start_client() -> void:
	if waiting:
		return
	waiting = true
	var err := network.create_client(ip, port)
	if err != OK:
		print("Encountered other error in gateway: ", err)
		email_held = ""
		username_held = ""
		password_held = ""
		other_error.emit(err)
		return
	
	multiplayer.multiplayer_peer = network
	print("Gateway client started")

func reset_wait() -> void:
	time = 0.
	waiting = false

func _peer_connected(net_id: int) -> void:
	reset_wait()
	print("Connected to Gateway server: ", net_id)
	succeeded = false
	call_deferred("send_held_data")

func _peer_disconnected(net_id: int) -> void:
	print("Disconnected from Gateway server")
	network.close()
	if not succeeded:
		print("And timed out")
		timeout.emit()

@rpc("any_peer", "call_remote", "reliable", 0)
func log_in(authenticated: bool) -> void:
	#succeeded = true
	#network.disconnect_peer(1)
	if authenticated:
		print("Successful login")
		success.emit()
	else:
		print("Failed login")
		network.disconnect_peer(1)
		invalid.emit()

@rpc("any_peer", "call_remote", "reliable", 0)
func create_account(valid: bool) -> void:
	succeeded = true
	network.disconnect_peer(1)
	if valid:
		print("Successful account creation")
	else:
		print("Failed account creation")
	creation_status.emit(valid)

@rpc("any_peer", "call_remote", "reliable", 1)
func get_server_list(servers: Array[Dictionary]) -> void:
	got_server_list.emit(servers)

@rpc("any_peer", "call_remote", "reliable", 1)
func join_server(address: String, port: int, token: String) -> void:
	succeeded = true
	await get_tree().create_timer(0.5).timeout
	joined_server.emit(address, port, token)
