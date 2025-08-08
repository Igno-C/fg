extends ServerConnector

#var network = ENetMultiplayerPeer.new()
#var port: int
#var max_users: int
## The token used to verify servers trying to connect to gateway server
#var auth_token: String

var config := ConfigFile.new()

var servers: Dictionary[int, ServerStats] = {}

func _ready() -> void:
	config.load("res://gateway.cfg")
	var port: int = config.get_value("ServerList", "port")
	var max_servers: int = config.get_value("ServerList", "max_servers")
	var auth_token: String = config.get_value("ServerList", "auth_token")
	
	set_name("serverlist")
	set_target_name("game server")
	set_token(auth_token)
	set_server(port, max_servers)
	
	start_server()
	multiplayer.peer_connected.connect(_peer_connected)
	multiplayer.peer_disconnected.connect(_peer_disconnected)

func get_server_list() -> Array[Dictionary]:
	var list: Array[Dictionary]
	list.assign(servers.values().map(func(s: ServerStats): return s.to_dict()))
	
	return list

func join_server(name: String, pid: int) -> ServerDirections:
	if servers.is_empty():
		return ServerDirections.new("", -2, "")
	for net_id in servers:
		var server = servers[net_id]
		if server.name == name:
			randomize()
			var token := str(randi()).sha256_text()
			rpc_id(net_id, "update", token, pid)
			return ServerDirections.new(server.address, server.port, token)
	
	return ServerDirections.new("", -1, "")


func _peer_connected(user_id) -> void:
	servers[user_id] = ServerStats.new()

func _peer_disconnected(user_id) -> void:
	servers.erase(user_id)

@rpc("any_peer", "call_remote", "unreliable_ordered", 0)
func update(current_players: int, max_players: int = -1, port: int = -1, name: String = ""):
	var net_id := multiplayer.get_remote_sender_id()
	var server_stats: ServerStats = servers[net_id]
	server_stats.current_players = current_players
	if max_players != -1:
		var mult_peer = multiplayer.multiplayer_peer as ENetMultiplayerPeer
		var this_peer := mult_peer.get_peer(net_id)
		server_stats.address = this_peer.get_remote_address()
		server_stats.port = port
		server_stats.name = name
	#print(get_server_list())
