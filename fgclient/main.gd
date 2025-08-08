extends Node

@onready var debug_label: Label = get_node("Ui/DebugLabel")
@onready var ticktimer: Timer = get_node("TickTimer")
var player_controller: PlayerController = null
@onready var game_node: Node2D = get_node("Game")
@onready var ui_node: Control = get_node("Ui")

@onready var server: Server = get_node("/root/ServerNode")
@onready var gateway: GatewayServer = get_node("/root/GatewayServer")

var map: BaseMap = null

var player_net_id = 0
#var player: PlayerEntity = null

# Dictionary from net_ids to player entities
var players = {}

func _ready() -> void:
	ticktimer.timeout.connect(on_tick)
	
	server.player_update.connect(player_update)
	server.data_update.connect(data_update)
	server.net_id_update.connect(set_net_id)
	
	server.connection_failure.connect(_on_connection_failure)
	
	spawn_loginscreen()

func spawn_loginscreen() -> void:
	var loginscreen: PackedScene = load("uid://bveij4bkswqeh")
	var ls := loginscreen.instantiate()
	ls.login_success.connect(_on_login_success)
	ui_node.add_child(ls)

func _on_login_success(token: String, ip: String, port: int) -> void:
	ui_node.get_node("LoginScreen").queue_free()
	
	server.connect_to_server(token, ip, port)

func _on_connection_failure(err: int) -> void:
	print("Connection to game server failed with err ", err)
	

func spawn_player_controller() -> void:
	if player_controller != null:
		return
	var pc := load("res://scripts/player/player_controller.gd")
	print(typeof(pc))
	player_controller = pc.new()
	player_controller.set_debug_label.connect(set_debug_label)
	player_controller.send_move.connect(server.send_move)
	player_controller.send_interaction.connect(server.send_interaction)
	ticktimer.timeout.connect(player_controller.on_tick)
	game_node.add_child(player_controller)

func on_tick() -> void:
	#print("main tick")
	pass

func set_debug_label(text: String) -> void:
	debug_label.set_text(text)

func set_net_id(net_id: int) -> void:
	player_net_id = net_id

func player_update(x: int, y: int, speed: int, net_id: int) -> void:
	print("Player ", player_net_id, " got response: ", x, ", ", y, ", ", speed, " for net_id ", net_id)
	
	var p: PlayerEntity
	if not players.has(net_id):
		p = spawn_player(net_id)
	else:
		p = players[net_id]
	if net_id == player_net_id:
		player_controller.receive_move(x, y, speed)
	else:
		p.move(Vector2i(x, y), speed)
	
func data_update(data: PlayerContainer, net_id: int) -> void:
	print("Player ", player_net_id, " Got pdata: ", data, " for net_id ", net_id)
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
	if net_id == player_net_id:
		load_map(data.get_location())

# Doesn't do anything if given mapname is the same as loaded map
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
	spawn_player_controller()
	player_controller.map = map

func spawn_player(net_id: int) -> PlayerEntity:
	print("Player ", player_net_id, " Spawning new player entity for net_id ", net_id)
	var p: PlayerEntity
	if net_id == player_net_id:
		var playerscene: PackedScene = load("res://scenes/player/GenericPlayer.tscn")
		#var additionsscene: PackedScene = load("res://scenes/player/TheUserAdditions.tscn")
		var camera: Camera2D = Camera2D.new()
		p = playerscene.instantiate()
		p.add_child(camera)
		if player_controller == null:
			spawn_player_controller()
		player_controller.player = p
		player_controller.camera = camera
	else:
		var playerscene: PackedScene = load("res://scenes/player/GenericPlayer.tscn")
		p = playerscene.instantiate()
	players[net_id] = p
	game_node.add_child(p)
	
	return p

func despawn_player(net_id: int) -> void:
	players[net_id].queue_free()
	players.erase(net_id)
