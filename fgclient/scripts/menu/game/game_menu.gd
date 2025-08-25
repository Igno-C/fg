extends Control

@onready var chat: Control = get_node("Chat")

func _ready() -> void:
	ServerNode.got_chat.connect(_on_got_chat)

func _process(delta: float) -> void:
	if Input.is_action_just_pressed("ChatOpen"):
		chat.open_chat()

func _on_got_chat(from: String, text: String, is_dm: bool) -> void:
	chat._on_got_chat(from, text, is_dm)

func set_dm_target(username: String, target_pid: int) -> void:
	chat.set_dm_target(username, target_pid)
