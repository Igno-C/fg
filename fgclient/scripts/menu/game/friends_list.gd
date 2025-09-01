extends ColorRect

@onready var requests_list: VBoxContainer = $MarginContainer/VBox/RequestsVBox/ScrollContainer/RequestsList

var friend_statuses: Dictionary[String, String]

func _ready() -> void:
	requests_list.child_order_changed.connect(try_hide_requestlist)

func _on_friend_data_update(uname: String, server_name: String) -> void:
	friend_statuses[uname] = server_name
	redo_friends_list()

func redo_friends_list() -> void:
	var online_str := ""
	var offline_str := ""
	for uname in friend_statuses:
		uname = uname.strip_edges().replace("\n", " ").replace("[", "[lb]")
		var server := friend_statuses[uname]
		if server != "":
			online_str += "[color=#0097DD][b]%s[/b][/color] @ %s\n" % [uname, server]
		else:
			offline_str += "[color=#636363][b]%s[/b][/color]\n" % uname
	$MarginContainer/VBox/FriendsList.text = online_str + offline_str

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

func try_hide_requestlist(_node) -> void:
	if requests_list.get_child_count() == 0:
		$MarginContainer/VBox/RequestsVBox.visible = false

func accept_request(pid: int) -> void:
	var event := GenericEvent.friend_accept(pid)
	ServerNode.send_event(event)
