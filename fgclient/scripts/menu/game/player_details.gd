extends Control

@onready var player_stats: ColorRect = $PlayerStats
@onready var stat_grid: GridContainer = $%StatGrid
@onready var player_name: Label = $%PlayerName
@onready var equipped_slot: InventoryItem = $%EquippedSlot
@onready var friends_list: ColorRect = $FriendsList
@onready var invite_button: Button = $%InviteButton


var shown_pid: int = -1
var skill_nodes: Dictionary[String, Control] = {}
#var friend_requests: Array[int] = []
signal system_chat(text: String)

func _ready() -> void:
	var player_stat_scene: PackedScene = load("res://scenes/menu/game/PlayerStat.tscn")
	for skill in PlayerContainer.skill_array():
		var player_stat_node: Control = player_stat_scene.instantiate()
		stat_grid.add_child(player_stat_node)
		player_stat_node.set_skill(skill)
		skill_nodes[skill] = player_stat_node

func open_friends() -> void:
	player_stats.visible = false
	friends_list.visible = not friends_list.visible

func close_all() -> bool:
	var all_already_closed: bool = (not player_stats.visible) and (not friends_list.visible)
	player_stats.visible = false
	friends_list.visible = false
	return all_already_closed

func _on_friend_data_update(uname: String, server_name: String) -> void:
	friends_list._on_friend_data_update(uname, server_name)
	#friend_statuses[data.get_name()] = data.get_server_name()
	#redo_friends_list()

func _on_get_friend_request(pid: int, uname: String) -> void:
	print("Got friend request from %s." % uname)
	system_chat.emit("Got friend request from %s." % uname)
	friends_list.add_request(pid, uname)

func send_invite() -> void:
	var event := GenericEvent.friend_request(shown_pid)
	ServerNode.send_event(event)

func populate(data: PlayerContainer, show_invite_btn: bool) -> void:
	player_stats.visible = true
	
	for skill in PlayerContainer.skill_array():
		var level = data.get_stat(skill)
		var xp = data.get_stat_progress(skill)
		skill_nodes[skill].set_stats(level, xp)
	player_name.text = data.get_name()
	equipped_slot.set_item(data.get_equipped_item())
	invite_button.visible = show_invite_btn
	shown_pid = data.get_pid()

func _on_close_button_pressed() -> void:
	close_all()
