class_name GameManager
extends Node

#@onready var server: Server = get_node("/root/ServerNode")
#@onready var ticker: Timer = get_node("/root/Ticker")
@onready var game_node: Node2D = get_node("/root/Main/Game")

var player_controller: PlayerController = null
var map: BaseMap = null

var player_pid := -1
var player_friends: Array[int] = []
var players: Dictionary[int, PlayerEntity] = {}
var entities: Dictionary[int, GenericEntity] = {}

signal set_debug_label(text: String)
signal your_data_updated(data: PlayerContainer)
signal set_context_menu(menu: PopupPanel)
signal got_friend_request(pid: int, uname: String)
signal got_friend_update(uname: String, server: String)

func _ready() -> void:
	ServerNode.player_update.connect(_on_player_update)
	ServerNode.data_update.connect(_on_data_update)
	ServerNode.generic_response.connect(_on_generic_response)
	ServerNode.entity_update.connect(_on_entity_update)
	ServerNode.edata_update.connect(_on_edata_update)
	var friend_timer := Timer.new()
	friend_timer.autostart = true
	friend_timer.wait_time = 20.
	friend_timer.timeout.connect(get_friend_data)
	add_child(friend_timer)

func get_friend_data() -> void:
	for friend in player_friends:
		ServerNode.send_data_request(friend)

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
	
func _on_data_update(data: PlayerContainer) -> void:
	var pid = data.get_pid()
	print("pid %s got pdata" % pid)
	if player_friends.has(pid):
		got_friend_update.emit(data.get_name(), data.get_server_name())
	if pid == player_pid:
		if player_friends.is_empty():
			player_friends = data.get_friends()
			get_friend_data()
		else:
			player_friends = data.get_friends()
		your_data_updated.emit(data)
	if not players.has(pid):
		return
	var p := players[pid]
	p.receive_data(data)

func _on_entity_update(x: int, y: int, speed: int, data_version: int, entity_id: int) -> void:
	print("entity %s got update: %s, %s, %s, dataver %s" % [entity_id, x, y, speed, data_version])
	var e: GenericEntity
	if not entities.has(entity_id):
		e = spawn_entity(entity_id)
	else:
		e = entities[entity_id]
	
	if e.data_version < data_version:
		ServerNode.send_edata_request(x, y, entity_id)
		e.data_version = data_version
	
	e.move(Vector2i(x, y), speed)

func _on_edata_update(
	interactable: bool,
	walkable: bool,
	related_scene: String,
	data: Dictionary,
	entity_id: int
) -> void:
	print("entity_id %s got pdata" % entity_id)
	if related_scene.is_empty():
		despawn_entity(entity_id)
		return
	var e: GenericEntity
	if not entities.has(entity_id):
		spawn_entity(entity_id)
	e = entities[entity_id]
	
	e.interactable = interactable
	e.walkable = walkable
	e.load_scene(related_scene)
	e.receive_data(data)

func _on_generic_response(response: GenericResponse) -> void:
	match response.response_type():
		GenericResponse.RESPONSE_ERR:
			print("Got err generic response!")
		GenericResponse.RESPONSE_LOAD_MAP:
			print("Got load map generic response")
			load_map(response.as_load_map())
		GenericResponse.RESPONSE_GOT_FRIEND_REQUEST:
			var resp = response.as_got_friend_request()
			print("Got got friend request generic response: ", resp)
			got_friend_request.emit(resp["pid"], resp["name"])
		GenericResponse.RESPONSE_DESPAWN_PLAYER:
			var resp = response.as_despawn_player()
			despawn_player(resp)

func load_map(mapname: String):
	if map != null: map.queue_free()
	
	print("Loading map: ", mapname)
	for pid: int in players:
		if pid != player_pid:
			despawn_player(pid)
	for entity_id: int in entities:
		despawn_entity(entity_id)
	
	var mapscene: PackedScene = load("res://maps/%s.tscn" % mapname)
	map = mapscene.instantiate()
	map.name = mapname
	game_node.add_child(map)
	game_node.move_child(map, 0)
	if player_controller != null:
		player_controller.map = map

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
		if map != null:
			player_controller.map = map
	
	players[pid] = p
	game_node.add_child(p)
	
	return p

func despawn_player(pid: int) -> void:
	print("Despawning player with pid ", pid)
	players[pid].queue_free()
	players.erase(pid)

func spawn_entity(entity_id: int) -> GenericEntity:
	print("Spawning new generic entity for entity id %s" % entity_id)
	var e: GenericEntity = GenericEntity.new()
	e.entity_id = entity_id
	entities[entity_id] = e
	game_node.add_child(e)
	
	return e

func despawn_entity(entity_id: int) -> void:
	print("Despawning entity with entity id ", entity_id)
	entities[entity_id].queue_free()
	entities.erase(entity_id)

func spawn_player_controller() -> void:
	var pc: Script = load("res://scripts/player/player_controller.gd")
	player_controller = pc.new()
	player_controller.set_debug_label.connect(set_debug_label.emit)
	player_controller.send_move.connect(ServerNode.send_move)
	player_controller.send_event.connect(ServerNode.send_event)
	player_controller.open_context_at.connect(open_context_at)
	Ticker.timeout.connect(player_controller.on_tick)
	game_node.add_child(player_controller)

func open_context_at(pos: Vector2i) -> void:
	var players_at := get_players_at(pos)
	var entities_at := get_entities_at(pos)
	
	var context_menu_scene: PackedScene = load("res://scenes/menu/game/ContextMenu.tscn")
	var context_menu: ContextPopup = context_menu_scene.instantiate()
	context_menu.walk_to_pos.connect(player_controller.go_to_pos)
	context_menu.interact_with_entity.connect(interact_with_entity)
	context_menu.entities = entities_at
	context_menu.players = players_at
	context_menu.related_pos = pos
	context_menu.player_pid = player_pid
	set_context_menu.emit(context_menu)

func interact_with_entity(entity: GenericEntity) -> void:
	var entity_id := entity.entity_id
	var entity_pos := entity.pos
	
	var event := GenericEvent.interaction(entity_pos.x, entity_pos.y, entity_id)
	
	ServerNode.send_event(event)

func get_players_at(pos: Vector2i) -> Array[PlayerEntity]:
	var at_list: Array[PlayerEntity] = []
	for player: PlayerEntity in players.values():
		if player.pos == pos:
			at_list.push_back(player)
	return at_list

func get_entities_at(pos: Vector2i) -> Array[GenericEntity]:
	var at_list: Array[GenericEntity] = []
	for entity: GenericEntity in entities.values():
		if entity.pos == pos:
			at_list.push_back(entity)
	return at_list
