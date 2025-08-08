class_name Server

extends Node

var network: ENetMultiplayerPeer
var token := ""

signal player_update(x: int, y: int, speed: int, net_id: int)
signal entity_update(x: int, y: int, speed: int)
signal data_update(data: PlayerContainer, net_id: int)

signal net_id_update(net_id: int)

signal connection_success
signal connection_failure(err: int)

func _ready() -> void:
	var move_config: Dictionary = {
		"rpc_mode": MultiplayerAPI.RPCMode.RPC_MODE_AUTHORITY,
		"transfer_mode": MultiplayerPeer.TransferMode.TRANSFER_MODE_UNRELIABLE_ORDERED,
		"call_local": false,
		"channel": 0,
	}
	var data_config: Dictionary = {
		"rpc_mode": MultiplayerAPI.RPCMode.RPC_MODE_AUTHORITY,
		"transfer_mode": MultiplayerPeer.TransferMode.TRANSFER_MODE_RELIABLE,
		"call_local": false,
		"channel": 1,
	}
	var interaction_config: Dictionary = {
		"rpc_mode": MultiplayerAPI.RPCMode.RPC_MODE_AUTHORITY,
		"transfer_mode": MultiplayerPeer.TransferMode.TRANSFER_MODE_RELIABLE,
		"call_local": false,
		"channel": 2,
	}
	rpc_config("pmove", move_config)
	rpc_config("pdata", data_config)
	rpc_config("pinter", interaction_config)
	
	var mult = multiplayer as SceneMultiplayer
	mult.auth_callback = auth_callback
	mult.auth_timeout = 5.
	mult.peer_authenticating.connect(_on_peer_authenticating)
	mult.peer_authentication_failed.connect(_on_peer_authentication_failed)
	
	mult.peer_connected.connect(_peer_connected)
	mult.peer_disconnected.connect(_peer_disconnected)

func connect_to_server(t: String, ip: String, port: int) -> void:
	network = ENetMultiplayerPeer.new()
	#network.peer_connected.connect(_peer_connected)
	#network.peer_disconnected.connect(_peer_disconnected)
	
	if network.create_client(ip, port) != OK:
		print("pissed")
		connection_failure.emit(2)
	
	multiplayer.multiplayer_peer = network
	
	token = t

func auth_callback(net_id: int, data: PackedByteArray) -> void:
	var mult = multiplayer as SceneMultiplayer
	print("Received auth from server: ", data)
	mult.complete_auth(net_id)

func _on_peer_authenticating(net_id: int) -> void:
	var mult = multiplayer as SceneMultiplayer
	mult.send_auth(net_id, token.to_ascii_buffer())
	token = ""

func _on_peer_authentication_failed(net_id: int) -> void:
	connection_failure.emit(1)

func _process(delta: float) -> void:
	pass

func _peer_connected(_id: int) -> void:
	print("Successful connect as ", multiplayer.multiplayer_peer.get_unique_id())
	var net_id = multiplayer.multiplayer_peer.get_unique_id()
	net_id_update.emit(net_id)

func _peer_disconnected(_id: int) -> void:
	print("Disconnect as ", multiplayer.multiplayer_peer.get_unique_id())
	net_id_update.emit(0)

func send_move(x: int, y: int, speed: int) -> void:
	#var timestamp = Time.get_ticks_msec()
	#print("Fakelagging...")
	#var timer := Timer.new()
	#var fakelag: float = 0.15 + randf_range(-0.05, 0.05)
	#timer.autostart = true; timer.one_shot = true; timer.wait_time = fakelag
	#add_child(timer)
	#await timer.timeout
	#print("Actually sending now after ", Time.get_ticks_msec()-timestamp, "ms")
	rpc_id(1, "pmove", x, y, speed)

func send_interaction(x: int, y: int) -> void:
	print("Sending interaction at ", x, ", ", y)
	rpc_id(1, "pinter", x, y)

func pinter() -> void:
	pass

#@rpc("authority", "call_remote", "unreliable_ordered", 0)
func pmove(x: int, y: int, speed: int, net_id: int) -> void:
	player_update.emit(x, y, speed, net_id)

func pdata(data: PackedByteArray, net_id: int) -> void:
	var container: PlayerContainer = PlayerContainer.new()
	container.parse_bytearray(data)
	
	data_update.emit(container, net_id)
