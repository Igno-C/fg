class_name Server
extends Node

const FAKELAG_ENABLED: bool = false
const FAKELAG: float = 150.
const FAKEJITTER: float = 30.
var network: ENetMultiplayerPeer
var token := ""

signal player_update(x: int, y: int, speed: int, net_id: int)
#signal entity_update(x: int, y: int, speed: int)
signal generic_update()
signal data_update(data: PlayerContainer, net_id: int)

#signal net_id_update(net_id: int)

signal connection_success
signal connection_failure(err: String)

func _ready() -> void:
	var move_config: Dictionary = {
		"rpc_mode": MultiplayerAPI.RPCMode.RPC_MODE_AUTHORITY,
		"transfer_mode": MultiplayerPeer.TransferMode.TRANSFER_MODE_UNRELIABLE_ORDERED,
		"call_local": false,
		"channel": 0,
	}
	var data_config: Dictionary = {
		"rpc_mode": MultiplayerAPI.RPCMode.RPC_MODE_ANY_PEER,
		"transfer_mode": MultiplayerPeer.TransferMode.TRANSFER_MODE_RELIABLE,
		"call_local": false,
		"channel": 1,
	}
	var player_event_config: Dictionary = {
		"rpc_mode": MultiplayerAPI.RPCMode.RPC_MODE_ANY_PEER,
		"transfer_mode": MultiplayerPeer.TransferMode.TRANSFER_MODE_RELIABLE,
		"call_local": false,
		"channel": 2,
	}
	rpc_config("pmove", move_config)
	rpc_config("pdata", data_config)
	rpc_config("pevent", player_event_config)
	
	var mult = multiplayer as SceneMultiplayer
	mult.auth_callback = auth_callback
	mult.auth_timeout = 5.
	mult.peer_authenticating.connect(_on_peer_authenticating)
	mult.peer_authentication_failed.connect(_on_peer_authentication_failed)
	
	mult.peer_connected.connect(_on_peer_connected)
	mult.peer_disconnected.connect(_on_peer_disconnected)
	
	mult.connection_failed.connect(connection_failure.emit.bind("Error: Failed to connect to server"))

func connect_to_server(ip: String, port: int, t: String) -> void:
	network = ENetMultiplayerPeer.new()
	
	var err := network.create_client(ip, port)
	if err != OK:
		connection_failure.emit("Error: Failed to create network client - " + str(err))
	
	token = t
	multiplayer.multiplayer_peer = network

func auth_callback(net_id: int, data: PackedByteArray) -> void:
	var mult = multiplayer as SceneMultiplayer
	print("Received auth from server: ", data)
	mult.complete_auth(net_id)

func _on_peer_authenticating(net_id: int) -> void:
	var mult = multiplayer as SceneMultiplayer
	mult.send_auth(net_id, token.to_ascii_buffer())
	token = ""

func _on_peer_authentication_failed(net_id: int) -> void:
	connection_failure.emit("Error: Failed authentication with server")

func _process(delta: float) -> void:
	pass

func _on_peer_connected(net_id: int) -> void:
	print("Successful connect as ", net_id)
	connection_success.emit()

func _on_peer_disconnected(net_id: int) -> void:
	print("Disconnect as ", net_id)
	connection_failure.emit("Disconnected from server")

func send_move(x: int, y: int, speed: int) -> void:
	if FAKELAG_ENABLED:
		var timestamp := Time.get_ticks_msec()
		print("Fakelagging...")
		var fakelag: float = FAKELAG + randf_range(-FAKEJITTER, FAKEJITTER)
		await get_tree().create_timer(fakelag).timeout
		print("Actually sending now after ", Time.get_ticks_msec()-timestamp, "ms")
	rpc_id(1, "pmove", x, y, speed)

func send_event(event: GenericEvent) -> void:
	rpc_id(1, "pevent", event.to_bytearray())

func send_data_request(pid: int) -> void:
	rpc_id(1, "pdata", pid)

func pevent() -> void:
	pass

func pmove(x: int, y: int, speed: int, net_id: int) -> void:
	player_update.emit(x, y, speed, net_id)

func pdata(data: PackedByteArray, net_id: int) -> void:
	var container := PlayerContainer.from_bytearray(data)
	
	data_update.emit(container, net_id)
