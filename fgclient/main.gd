extends Node

@onready var debug_label: Label = get_node("Ui/DebugLabel")
@onready var ticker: Timer = get_node("/root/Ticker")
@onready var game_node: Node2D = get_node("Game")
@onready var ui_node: Control = get_node("Ui")

@onready var server: Server = get_node("/root/ServerNode")
@onready var gateway: GatewayServer = get_node("/root/GatewayServer")

var your_pid: int = -1
# Dictionary from net_ids to player entities

func _ready() -> void:
	ticker.timeout.connect(on_tick)
	
	server.connection_failure.connect(_on_connection_failure)
	server.connection_success.connect(_on_connection_success)
	
	spawn_loginscreen()
	#var pc: Script = load("res://scripts/player/player_controller.gd")
	#var player_controller: PlayerController = pc.new()

func spawn_loginscreen(with_err: String = "") -> void:
	var loginscreen: PackedScene = load("uid://bveij4bkswqeh")
	var ls := loginscreen.instantiate()
	ls.login_success.connect(_on_login_success)
	if not with_err.is_empty():
		ls.login_box.set_err(with_err)
	ui_node.add_child(ls)

func _on_login_success(pid: int, ip: String, port: int, token: String) -> void:
	server.connect_to_server(ip, port, token)
	your_pid = pid

func _on_connection_success() -> void:
	var manager := GameManager.new()
	manager.player_pid = your_pid
	manager.name = "GameManager"
	
	game_node.add_child(manager)
	
	ui_node.get_node("LoginScreen").queue_free()

func _on_connection_failure(err: String) -> void:
	for child in game_node.get_children():
		child.queue_free()
	
	spawn_loginscreen(err)

func on_tick() -> void:
	pass

func set_debug_label(text: String) -> void:
	debug_label.set_text(text)
