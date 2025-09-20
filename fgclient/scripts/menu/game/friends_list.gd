extends ColorRect

@onready var requests_list: VBoxContainer = $MarginContainer/VBox/RequestsVBox/ScrollContainer/RequestsList
@onready var friends_vbox: VBoxContainer = $MarginContainer/VBox/FriendsList/FriendsVBox

var friend_datas: Dictionary[int, PlayerContainer]
signal open_details_menu(data: PlayerContainer, invite_btn: bool, dm_btn: bool)

func _ready() -> void:
	requests_list.child_order_changed.connect(try_hide_requestlist)

func _on_friend_data_update(data: PlayerContainer) -> void:
	friend_datas[data.get_pid()] = data
	redo_friends_list()

func redo_friends_list() -> void:
	for node in friends_vbox.get_children():
		node.queue_free()
	var online_nodes: Array[Node] = []
	var offline_nodes: Array[Node] = []
	for data: PlayerContainer in friend_datas.values():
		var uname = data.get_name().strip_edges().replace("\n", " ").replace("[", "[lb]")
		var server := data.get_server_name()
		
		var newnode := RichTextLabel.new()
		newnode.bbcode_enabled = true
		newnode.custom_minimum_size.y = 25.
		
		if server != "":
			newnode.text = "[color=#0097DD][b]%s[/b][/color] @ %s\n" % [uname, server]
			newnode.gui_input.connect(friend_clicked.bind(data, true))
			offline_nodes.push_back(newnode)
		else:
			newnode.text = "[color=#636363][b]%s[/b][/color]\n" % uname
			newnode.gui_input.connect(friend_clicked.bind(data, false))
			online_nodes.push_back(newnode)
	for offline in offline_nodes:
		friends_vbox.add_child(offline)
	for online in online_nodes:
		friends_vbox.add_child(online)

func friend_clicked(event: InputEvent, data: PlayerContainer, dmable: bool) -> void:
	if event is InputEventMouseButton:
		if event.button_index == MOUSE_BUTTON_LEFT and event.pressed:
			open_details_menu.emit(data, false, dmable)

func add_request(pid: int, uname: String) -> void:
	$MarginContainer/VBox/RequestsVBox.visible = true
	var hbox := GridContainer.new()
	hbox.columns = 3
	var label := Label.new()
	label.text = uname
	hbox.add_child(label)
	var accept_button := Button.new()
	accept_button.text = "Accept"
	accept_button.pressed.connect(accept_request.bind(pid))
	accept_button.pressed.connect(hbox.queue_free)
	hbox.add_child(accept_button)
	
	var deny_button := Button.new()
	deny_button.text = "X"
	deny_button.pressed.connect(hbox.queue_free)
	hbox.add_child(deny_button)
	requests_list.add_child(hbox)

func try_hide_requestlist() -> void:
	if requests_list.get_child_count() == 0:
		$MarginContainer/VBox/RequestsVBox.visible = false

func deny_all_requests() -> void:
	for child in requests_list.get_children():
		child.queue_free()
	$MarginContainer/VBox/RequestsVBox.visible = false

func accept_request(pid: int) -> void:
	var event := GenericEvent.friend_accept(pid)
	ServerNode.send_event(event)
