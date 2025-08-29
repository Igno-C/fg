extends Node

const FAKELAG_ENABLED: bool = false
const FAKELAG: float = 150.
const FAKEJITTER: float = 30.
var network: ENetMultiplayerPeer
var token := ""

signal player_update(x: int, y: int, speed: int, data_version: int, pid: int)
signal entity_update(x: int, y: int, speed: int, data_version: int, entity_id: int)
signal generic_response(event: GenericResponse)
signal data_update(data: PlayerContainer, pid: int)
signal edata_update(interactable: bool, walkable: bool, related_scene: String, data: Dictionary, entity_id: int)
signal got_chat(from: String, text: String, is_dm: bool)

signal connection_success
signal connection_failure(err: String)

func _ready() -> void:
	# If done using @rpc instead of this way, doesn't work with godot-rust configs for some reason
	rpc_config("pmove", pmove_config)
	rpc_config("pdata", pdata_config)
	rpc_config("pevent", pevent_config)
	rpc_config("pchat", pchat_config)
	rpc_config("emove", emove_config)
	rpc_config("edata", edata_config)
	
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
	print("Sending move")
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

func send_dm(text: String, target_pid: int) -> void:
	rpc_id(1, "pchat", text, target_pid)

func send_zone_chat(text: String) -> void:
	rpc_id(1, "pchat", text, -1)

func send_edata_request(x: int, y: int, entity_id: int) -> void:
	rpc_id(1, "edata", x, y, entity_id)

func pevent(response_bytes: PackedByteArray) -> void:
	var response := GenericResponse.from_bytearray(response_bytes)
	generic_response.emit(response)

func pmove(x: int, y: int, speed: int, data_version: int, pid: int) -> void:
	player_update.emit(x, y, speed, data_version, pid)

func pdata(data: PackedByteArray) -> void:
	var container := PlayerContainer.from_bytearray(data)
	var pid: int = container.get_pid()
	
	data_update.emit(container, pid)

func pchat(from: String, text: String, is_dm: bool) -> void:
	got_chat.emit(from, text, is_dm)

func emove(x: int, y: int, speed: int, data_version: int, entity_id: int) -> void:
	entity_update.emit(x, y, speed, data_version, entity_id)

func edata(interactable: bool, walkable: bool, related_scene: String, data: Dictionary, entity_id: int) -> void:
	edata_update.emit(interactable, walkable, related_scene, data, entity_id)

# Networking configs for the RPCs
const pmove_config: Dictionary = {
	"rpc_mode": MultiplayerAPI.RPCMode.RPC_MODE_AUTHORITY,
	"transfer_mode": MultiplayerPeer.TransferMode.TRANSFER_MODE_UNRELIABLE_ORDERED,
	"call_local": false,
	"channel": 0,
}
const pdata_config: Dictionary = {
	"rpc_mode": MultiplayerAPI.RPCMode.RPC_MODE_ANY_PEER,
	"transfer_mode": MultiplayerPeer.TransferMode.TRANSFER_MODE_RELIABLE,
	"call_local": false,
	"channel": 1,
}
const pevent_config: Dictionary = {
	"rpc_mode": MultiplayerAPI.RPCMode.RPC_MODE_ANY_PEER,
	"transfer_mode": MultiplayerPeer.TransferMode.TRANSFER_MODE_RELIABLE,
	"call_local": false,
	"channel": 2,
}
const pchat_config: Dictionary = {
	"rpc_mode": MultiplayerAPI.RPCMode.RPC_MODE_ANY_PEER,
	"transfer_mode": MultiplayerPeer.TransferMode.TRANSFER_MODE_RELIABLE,
	"call_local": false,
	"channel": 3
}
const emove_config: Dictionary = {
	"rpc_mode": MultiplayerAPI.RPCMode.RPC_MODE_AUTHORITY,
	"transfer_mode": MultiplayerPeer.TransferMode.TRANSFER_MODE_RELIABLE,
	"call_local": false,
	"channel": 4
}
const edata_config: Dictionary = {
	"rpc_mode": MultiplayerAPI.RPCMode.RPC_MODE_ANY_PEER,
	"transfer_mode": MultiplayerPeer.TransferMode.TRANSFER_MODE_RELIABLE,
	"call_local": false,
	"channel": 4
}
