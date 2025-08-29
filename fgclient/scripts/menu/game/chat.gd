extends Control

@onready var chat_box: Control = get_node("ChatBox")
@onready var chat: RichTextLabel = get_node("ChatBox/VBox/ChatText")
@onready var line_edit: LineEdit = get_node("ChatBox/VBox/LineEdit")
@onready var hide_button: Button = get_node("HideButton")

const MAX_CHAT_MESSAGES: int = 80

var dm_target: int = -1
var dm_username: String
var is_dming: bool = false

func _ready() -> void:
	set_zone_chat()

func _process(delta: float) -> void:
	if line_edit.has_focus():
		if Input.is_action_just_pressed("ChatSwitchMode"):
			is_dming = not is_dming
			if dm_target == -1:
				is_dming = false
			set_hint()

func set_hint() -> void:
	if is_dming:
		line_edit.placeholder_text = "Messaging %s" % dm_username
	else:
		line_edit.placeholder_text = "Zone chat"

func set_dm_target(username: String, target_pid: int) -> void:
	dm_target = target_pid
	dm_username = username
	is_dming = true
	set_hint()

func set_zone_chat() -> void:
	is_dming = false
	set_hint()

func open_chat() -> void:
	chat_box.visible = true
	line_edit.grab_focus()

func push_chat_message(from: String, text: String, is_dm: bool) -> void:
	# Prevents showing newlines, escapes BBCode injection
	var message: String
	if from.is_empty():
		message = "\n" + text
	else:
		from = from.strip_edges().replace("\n", " ").replace("[", "[lb]")
		text = text.strip_edges().replace("\n", " ").replace("[", "[lb]")
		if is_dm:
			message = "\n[color=#6996FF][b][%s]: [/b]%s[/color]" % [from, text]
		else:
			message = "\n[b]<%s>: [/b] %s" % [from, text]
	chat.text += message
	if chat.get_line_count() > MAX_CHAT_MESSAGES:
		chat.text = trim_first_line(chat.text)

func trim_first_line(string: String) -> String:
	var line_end_index = string.find("\n", 1)
	return string.substr(line_end_index, string.length() - line_end_index)

func _on_got_chat(from: String, text: String, is_dm: bool) -> void:
	push_chat_message(from, text, is_dm)

func _on_hide_pressed() -> void:
	chat_box.visible = not chat_box.visible
	
	if chat_box.visible:
		hide_button.text = "←"
	else:
		hide_button.text = "→"

func _on_chat_send(text: String) -> void:
	line_edit.text = ""
	line_edit.release_focus()
	text = text.strip_edges()
	if text.is_empty():
		return
	if dm_target == -1:
		ServerNode.send_zone_chat(text)
	else:
		ServerNode.send_dm(text, dm_target)

func _on_chat_text_focus(focus: bool) -> void:
	chat.scroll_following = not focus
