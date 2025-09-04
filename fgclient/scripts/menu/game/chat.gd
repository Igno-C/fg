extends Control

@onready var chat_box: Control = $ChatBox
@onready var chat_scroll: ScrollContainer = $ChatBox/VBox/ChatScroll
@onready var line_edit: LineEdit = $ChatBox/VBox/LineEdit
@onready var hide_button: Button = $HideButton

const MAX_CHAT_MESSAGES: int = 80

var dm_target: int = -1
var dm_username: String
var is_dming: bool = false

var message_nodes: Array[RichTextLabel] = []

func _ready() -> void:
	set_zone_chat()
	line_edit.gui_input.connect(_on_line_edit_gui_input)

func _process(delta: float) -> void:
	if chat_scroll.follow_focus:
		chat_scroll.scroll_vertical = chat_scroll.get_v_scroll_bar().max_value

func _on_line_edit_gui_input(event: InputEvent) -> void:
	if event.is_action_pressed("ChatSwitchMode"):
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
	open_chat()

func set_zone_chat() -> void:
	is_dming = false
	set_hint()

func open_chat() -> void:
	chat_box.visible = true
	line_edit.grab_focus()

func system_chat(text: String) -> void:
	push_chat_message(text, "", -1, false)

func push_chat_message(text: String, from: String, from_pid: int, is_dm: bool) -> void:
	# Prevents showing newlines, escapes BBCode injection
	var message: String
	if from.is_empty():
		message = text
	else:
		from = from.strip_edges().replace("\n", " ").replace("[", "[lb]")
		text = text.strip_edges().replace("\n", " ").replace("[", "[lb]")
		if is_dm:
			message = "[color=#6996FF][b][%s]: [/b]%s[/color]" % [from, text]
		else:
			message = "[b]<%s>: [/b] %s" % [from, text]
	var messagebox := RichTextLabel.new()
	messagebox.text = message
	messagebox.scroll_active = false
	messagebox.bbcode_enabled = true
	messagebox.fit_content = true
	messagebox.focus_mode = Control.FOCUS_CLICK
	messagebox.focus_entered.connect(_on_chat_text_focus.bind(true))
	messagebox.focus_exited.connect(_on_chat_text_focus.bind(false))
	
	if from_pid != -1:
		messagebox.gui_input.connect(_on_message_clicked.bind(from, from_pid))
	
	$ChatBox/VBox/ChatScroll/VBox.add_child(messagebox)
	message_nodes.push_back(messagebox)
	
	if message_nodes.size() > MAX_CHAT_MESSAGES:
		var message_node: RichTextLabel = message_nodes.pop_front()
		message_node.queue_free()

func trim_first_line(string: String) -> String:
	var line_end_index = string.find("\n", 1)
	return string.substr(line_end_index, string.length() - line_end_index)

func _on_message_clicked(event: InputEvent, from: String, from_pid: int) -> void:
	if event is InputEventMouseButton:
		if event.button_index == MOUSE_BUTTON_RIGHT and event.pressed:
			if from_pid != -1:
				set_dm_target(from, from_pid)

func _on_got_chat(text: String, from: String, from_pid: int, is_dm: bool) -> void:
	push_chat_message(text, from, from_pid, is_dm)

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
		push_chat_message(text, "To " + dm_username, dm_target, true)

func _on_chat_text_focus(focus: bool) -> void:
	#chat.scroll_following = not focus
	chat_scroll.follow_focus = not focus
