class_name GameManager
extends Node

@onready var server: Server = get_node("/root/ServerNode")
@onready var ticker: Timer = get_node("/root/Ticker")
@onready var game_node: Node2D = get_node("/root/Main/Game")

var player_controller: PlayerController = null
var map: BaseMap = null

var player_pid = -1
var players: Dictionary[int, PlayerEntity] = {}

signal set_debug_label(text: String)

func _ready() -> void:
	server.player_update.connect(_on_player_update)
	server.data_update.connect(_on_data_update)
	server.generic_update.connect(_on_generic_update)

func _on_player_update(x: int, y: int, speed: int, pid: int) -> void:
	#print("Player ", player_net_id, " got response: ", x, ", ", y, ", ", speed, " for net_id ", net_id)
	print("pid %s got update: %s, %s, %s" % [pid, x, y, speed])
	var p: PlayerEntity
	# Gets player or spawns if new
	if not players.has(pid):
		p = spawn_player(pid)
	else:
		p = players[pid]
	
	# Forwards the move info to the given player
	# or to the controller if it's the user player
	if pid == player_pid:
		player_controller.receive_move(x, y, speed)
	else:
		p.move(Vector2i(x, y), speed)
	
func _on_data_update(data: PlayerContainer, net_id: int) -> void:
	#print("Player ", player_net_id, " Got pdata: ", data, " for net_id ", net_id)
	print("net_id %s got pdata" % net_id)
	if data.is_null():
		despawn_player(net_id)
		return
	var p: PlayerEntity
	if not players.has(net_id):
		p = spawn_player(net_id)
		p.position.x = -2000.0
		p.position.y = -2000.0
	else:
		p = players[net_id]
	p.receive_data(data)
	if net_id == player_pid:
		load_map(data.get_location())

func _on_generic_update(response: GenericResponse) -> void:
	pass

#func _on_entity_update() -> void:
	#pass


func load_map(mapname: String):
	# There is a map loaded and there is no need to change it
	if map != null and map.name == mapname:
		return
	
	if map != null: map.queue_free()
	
	print("Loading map: ", mapname)
	for net_id: int in players:
		var p: PlayerEntity = players[net_id]
		if p.data.get_location() != mapname:
			p.queue_free()
			players.erase(net_id)
	
	var mapscene: PackedScene = load("res://maps/" + mapname + ".tscn")
	map = mapscene.instantiate()
	map.name = mapname
	game_node.add_child(map)
	#spawn_player_controller()
	player_controller.map = map

func spawn_player(pid: int) -> PlayerEntity:
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
	
	return p

func despawn_player(net_id: int) -> void:
	players[net_id].queue_free()
	players.erase(net_id)

func spawn_player_controller() -> void:
	var pc: Script = load("res://scripts/player/player_controller.gd")
	player_controller = pc.new()
	player_controller.set_debug_label.connect(set_debug_label.emit)
	player_controller.send_move.connect(server.send_move)
	player_controller.send_event.connect(server.send_event)
	ticker.timeout.connect(player_controller.on_tick)
	game_node.add_child(player_controller)
