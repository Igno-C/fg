class_name GameMenu
extends Control

@onready var chat: Control = get_node("Chat")
@onready var inventory: Control = get_node("Inventory")
@onready var player_details: Control = get_node("PlayerDetails")

var context_menu: ContextPopup = null

signal swap_items(from: int, to: int)

## Set to null to close
func set_context_menu(new_menu: ContextPopup) -> void:
	if context_menu != null:
		context_menu.queue_free()
	context_menu = new_menu
	if context_menu != null:
		context_menu.inspect_player.connect(inspect_player)
		add_child(context_menu)

func _ready() -> void:
	ServerNode.got_chat.connect(_on_got_chat)

func _process(delta: float) -> void:
	if Input.is_action_just_pressed("ChatOpen"):
		chat.open_chat()

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("InventoryOpen"):
		inventory.toggle()

func _on_got_chat(from: String, text: String, is_dm: bool) -> void:
	chat._on_got_chat(from, text, is_dm)

func inspect_player(data: PlayerContainer) -> void:
	player_details.populate(data)

func update_inventory(data: PlayerContainer) -> void:
	inventory.populate(data)

func set_dm_target(username: String, target_pid: int) -> void:
	chat.set_dm_target(username, target_pid)
