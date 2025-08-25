class_name GameManager
extends Node

#@onready var server: Server = get_node("/root/ServerNode")
@onready var ticker: Timer = get_node("/root/Ticker")
@onready var game_node: Node2D = get_node("/root/Main/Game")

var player_controller: PlayerController = null
var map: BaseMap = null

var player_pid = -1
var players: Dictionary[int, PlayerEntity] = {}
var entities: Dictionary[int, GenericEntity] = {}
#var player_datas: Dictionary[int, PlayerContainer] = {}

signal set_debug_label(text: String)

func _ready() -> void:
	ServerNode.player_update.connect(_on_player_update)
	ServerNode.data_update.connect(_on_data_update)
	ServerNode.generic_response.connect(_on_generic_response)
	#server.entity_update.connect(_on_entity_update)

func _on_player_update(x: int, y: int, speed: int, data_version: int, pid: int) -> void:
	print("pid %s got update: %s, %s, %s, dataver %s" % [pid, x, y, speed, data_version])
	var p: PlayerEntity
	if not players.has(pid):
		p = spawn_player(pid)
	else:
		p = players[pid]
	
	if p.data_version < data_version:
		ServerNode.send_data_request(pid)
		p.data_version = data_version
	
	if pid == player_pid:
		player_controller.receive_move(x, y, speed)
	else:
		p.move(Vector2i(x, y), speed)
	
func _on_data_update(data: PlayerContainer, pid: int) -> void:
	#print("Player ", player_net_id, " Got pdata: ", data, " for net_id ", net_id)
	print("pid %s got pdata" % pid)
	if data.is_null():
		despawn_player(pid)
		return
	var p: PlayerEntity
	if not players.has(pid):
		spawn_player(pid)
	p = players[pid]
	p.receive_data(data)

func _on_generic_response(response: GenericResponse) -> void:
	match response.response_type():
		GenericResponse.RESPONSE_ERR:
			print("Got err generic response!")
		GenericResponse.RESPONSE_LOAD_MAP:
			load_map(response.as_load_map())

func load_map(mapname: String):
	if map != null: map.queue_free()
	
	print("Loading map: ", mapname)
	for pid: int in players:
		if pid != player_pid:
			var p: PlayerEntity = players[pid]
			p.queue_free()
			players.erase(pid)
	
	var mapscene: PackedScene = load("res://maps/%s.tscn" % mapname)
	map = mapscene.instantiate()
	map.name = mapname
	game_node.add_child(map)
	player_controller.map = map
	player_controller.player.data.set_location(mapname)

func spawn_player(pid: int, data: PlayerContainer = null) -> PlayerEntity:
	print("Spawning new player entity for pid %s" % pid)
	var playerscene: PackedScene = load("res://scenes/player/GenericPlayer.tscn")
	var p: PlayerEntity = playerscene.instantiate()
	if pid == player_pid:
		var camera: Camera2D = Camera2D.new()
		p.add_child(camera)
		if player_controller == null:
			spawn_player_controller()
		player_controller.player = p
		player_controller.camera = camera
	
	players[pid] = p
	game_node.add_child(p)
	
	#if data != null:
		#p.receive_data(data)
	#else:
		#server.send_data_request(pid)
	
	return p

func despawn_player(net_id: int) -> void:
	players[net_id].queue_free()
	players.erase(net_id)

func spawn_player_controller() -> void:
	var pc: Script = load("res://scripts/player/player_controller.gd")
	player_controller = pc.new()
	player_controller.set_debug_label.connect(set_debug_label.emit)
	player_controller.send_move.connect(ServerNode.send_move)
	player_controller.send_event.connect(ServerNode.send_event)
	ticker.timeout.connect(player_controller.on_tick)
	game_node.add_child(player_controller)
