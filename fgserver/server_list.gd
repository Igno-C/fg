extends ServerConnector

# The gateway is updated on player counts every this many ticks
const everythismany: int = 60
var tick: int = 0
var server_name: String
var server_public_address: String

@onready var server_node: Server = get_node("/root/ServerNode")

func _ready() -> void:
	var config := ConfigFile.new()
	config.load("res://serverconfig.cfg")
	var ip: String = config.get_value("ServerList", "address")
	var port: int = config.get_value("ServerList", "port")
	var auth_token: String = config.get_value("ServerList", "auth_token")
	server_name = config.get_value("ServerList", "server_name", "unset server name")
	server_public_address = config.get_value("ServerList", "server_public_address")
	
	var game_manager: GameManager = get_node("/root/ManagerNode")
	game_manager.set_server_name(server_name)
	
	set_name("serverlist")
	set_target_name("gateway")
	set_token(auth_token)
	set_auto_reconnect(true)
	set_client(port, ip)
	
	start_server()
	multiplayer.peer_connected.connect(_on_peer_connected)

func _process(_delta: float) -> void:
	tick += 1
	if tick == everythismany:
		tick = 0
		if is_connected():
			update_gateway()

func update_gateway() -> void:
	var current_players := server_node.current_players()
	var max_players := server_node.max_players()
	var realport := server_node.port()
	var real_address := server_public_address
	rpc_id(1, "_update", current_players, max_players, realport, real_address, server_name)

func _on_peer_connected(_net_id: int) -> void:
	tick = everythismany-1

@rpc("any_peer", "call_remote", "reliable", 0)
func _update(token: String, pid: int):
	print("Received connection token from gateway for pid ", pid)
	server_node.register_token(token, pid)
