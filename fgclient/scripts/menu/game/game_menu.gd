class_name GameMenu
extends Control

@onready var chat: Control = get_node("Chat")
@onready var inventory: Control = get_node("Inventory")
@onready var player_details: Control = get_node("PlayerDetails")
@onready var big_menu: Control = get_node("BigMenu")

var context_menu: ContextPopup = null

signal system_chat(text: String)
signal disconnect_pressed

## Set to null to close
func set_context_menu(new_menu: ContextPopup) -> void:
	if context_menu != null:
		context_menu.queue_free()
	context_menu = new_menu
	if context_menu != null:
		context_menu.inspect_player.connect(player_details.populate)
		add_child(context_menu)

func _ready() -> void:
	ServerNode.got_chat.connect(_on_got_chat)
	system_chat.connect(chat.system_chat)
	player_details.system_chat.connect(system_chat.emit)
	player_details.set_dm_target.connect(chat.set_dm_target)
	
	$BigMenu/VBox/DisconnectButton.pressed.connect(ServerNode.disconnect_from_server)

func _process(delta: float) -> void:
	if Input.is_action_just_pressed("ChatOpen"):
		chat.open_chat()

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("InventoryOpen"):
		inventory.toggle()
	elif event.is_action_pressed("FriendsList"):
		player_details.open_friends()
	elif event.is_action_pressed("Close"):
		if context_menu != null:
			context_menu.queue_free()
		if player_details.close_all():
			big_menu.visible = not big_menu.visible

func _on_friend_data_update(uname: String, server_name: String) -> void:
	player_details._on_friend_data_update(uname, server_name)

func _on_get_friend_request(pid: int, uname: String) -> void:
	player_details._on_get_friend_request(pid, uname)

func _on_got_chat(text: String, from: String, from_pid: int, is_dm: bool) -> void:
	chat._on_got_chat(text, from, from_pid, is_dm)

func update_inventory(data: PlayerContainer) -> void:
	inventory.populate(data)

func set_dm_target(username: String, target_pid: int) -> void:
	chat.set_dm_target(username, target_pid)
