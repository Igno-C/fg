extends Node

@onready var debug_label: Label = get_node("DebugLabel")
@onready var ticker: Timer = get_node("/root/Ticker")
@onready var game_node: Node2D = get_node("Game")
@onready var ui_node: CanvasLayer = get_node("Ui")

#@onready var server: Server = get_node("/root/ServerNode")
@onready var gateway: GatewayServer = get_node("/root/GatewayServer")

var your_pid: int = -1
# Dictionary from net_ids to player entities

func _ready() -> void:
	ticker.timeout.connect(on_tick)
	
	ServerNode.connection_failure.connect(_on_connection_failure)
	ServerNode.connection_success.connect(_on_connection_success)
	
	spawn_loginscreen()
	#_on_connection_success()

func spawn_loginscreen(with_err: String = "") -> void:
	var loginscreen: PackedScene = load("uid://bveij4bkswqeh")
	var ls := loginscreen.instantiate()
	ls.login_success.connect(_on_login_success)
	for child in ui_node.get_children():
		child.queue_free()
	ui_node.add_child(ls)
	if not with_err.is_empty():
		ls.login_box.set_err(with_err)

func _on_login_success(pid: int, ip: String, port: int, token: String) -> void:
	ServerNode.connect_to_server(ip, port, token)
	your_pid = pid

# Setting up the nodes for actual gameplay happens here
func _on_connection_success() -> void:
	var manager := GameManager.new()
	manager.player_pid = your_pid
	manager.name = "GameManager"
	manager.set_debug_label.connect(set_debug_label)
	
	var game_menu_scene: PackedScene = load("res://scenes/menu/game/GameMenu.tscn")
	var game_menu: GameMenu = game_menu_scene.instantiate()
	game_menu.name = "GameMenu"
	
	manager.your_data_updated.connect(game_menu.update_inventory)
	manager.set_context_menu.connect(game_menu.set_context_menu)
	
	game_node.add_child(manager)
	ui_node.add_child(game_menu)
	
	ui_node.get_node("LoginScreen").queue_free()

func _on_connection_failure(err: String) -> void:
	for child in game_node.get_children():
		child.queue_free()
	
	spawn_loginscreen(err)

func on_tick() -> void:
	pass

func set_debug_label(text: String) -> void:
	if text.is_empty():
		return
	debug_label.text = text
