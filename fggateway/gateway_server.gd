extends ServerConnector

const TIMEOUT: float = 5.

@onready var auth_server: AuthServer = get_node("/root/AuthServer")
@onready var server_list: ServerList = get_node("/root/ServerList")
@onready var gateway_db_server: GatewayDbServer = get_node("/root/GatewayDbServer")

var waitinglist: Dictionary[int, GatewayWaiter] = {}


func _ready() -> void:
	var config := ConfigFile.new()
	config.load("res://gateway.cfg")
	var port: int = config.get_value("Gateway", "port")
	var max_users: int = config.get_value("Gateway", "max_users")
	set_name("gateway")
	set_target_name("game")
	set_server(port, max_users)
	
	start_server()
	
	multiplayer.peer_connected.connect(_on_peer_connected)
	multiplayer.peer_disconnected.connect(_on_peer_disconnected)
	
	auth_server.response.connect(_on_auth_response)
	auth_server.created_response.connect(_on_account_create_response)

func _process(delta: float) -> void:
	for net_id in waitinglist:
		var waiter: GatewayWaiter = waitinglist[net_id]
		if waiter.tick_timeout(delta):
			multiplayer.multiplayer_peer.disconnect_peer(net_id)

func _on_peer_connected(user_id) -> void:
	#print("User " + str(user_id) + " connected.")
	waitinglist[user_id] = GatewayWaiter.new()

func _on_peer_disconnected(user_id) -> void:
	#print("User " + str(user_id) + " disconnected.")
	waitinglist.erase(user_id)

func _on_auth_response(net_id: int, pid: int) -> void:
	if pid != -1:
		var waiter: GatewayWaiter = waitinglist[net_id]
		waiter.authenticate(pid)
		rpc_id(net_id, "log_in", true)
		gateway_db_server.create_new_player(pid, waiter.username)
	else:
		print("Login failed for ", net_id)
		rpc_id(net_id, "log_in", false)
		waitinglist[net_id].time = 0.

func _on_account_create_response(net_id: int, valid: bool) -> void:
	if valid:
		print("Created account for ", net_id)
	else:
		print("Failed account creation for ", net_id)
	rpc_id(net_id, "create_account", valid)
	waitinglist[net_id].time = 0.

@rpc("any_peer", "call_remote", "reliable", 0)
func log_in(username: String, password: String) -> void:
	var net_id := multiplayer.get_remote_sender_id()
	print("Received credentials from ", net_id)
	if waitinglist[net_id].received_request():
		auth_server.authenticate(net_id, username, password)
	else:
		print("Login attempted with queued request by ", net_id)

@rpc("any_peer", "call_remote", "reliable", 0)
func create_account(username: String, password: String) -> void:
	var net_id := multiplayer.get_remote_sender_id()
	print("Attempting account creation for ", net_id)
	var waiter := waitinglist[net_id]
	if waiter.received_request():
		waiter.username = username
		auth_server.create_account(net_id, username, password)
	else:
		print("Account creation attempted with queued request by ", net_id)

@rpc("any_peer", "call_remote", "reliable", 1)
func get_server_list() -> void:
	var net_id := multiplayer.get_remote_sender_id()
	var waiter: GatewayWaiter = waitinglist[net_id]
	if not waiter.authenticated:
		print("Get server list attempt by unauthenticated peer ", net_id)
		return
	print("Getting server list for ", net_id)
	var list: Array[Dictionary] = server_list.get_server_list()
	#var list: Array[Dictionary] = [{"name": "Server 1", "load": 1}, {"name": "Server 2", "load": 2}]
	rpc_id(net_id, "get_server_list", list)

@rpc("any_peer", "call_remote", "reliable", 1)
func join_server(server_name: String) -> void:
	var net_id := multiplayer.get_remote_sender_id()
	var waiter: GatewayWaiter = waitinglist[net_id]
	
	if not waiter.authenticated:
		print("Join server attempt by unauthenticated peer ", net_id)
		return
	
	var params = server_list.join_server(server_name, waiter.pid)
	print("Attempting to join user ", net_id, " to server ", server_name)
	print(params)
	rpc_id(net_id, "join_server", params.ip, params.port, params.token)
