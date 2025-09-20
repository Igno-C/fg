class_name ContextPopup
extends Panel

@onready var vbox: VBoxContainer = get_node("VBox")

signal inspect_player(data: PlayerContainer, show_invite_btn: bool, show_dm_button: bool)
signal interact_with_entity(entity_id: int)
signal walk_to_pos(pos: Vector2i)

var related_pos: Vector2i
var players: Array[PlayerEntity]
var entities: Array[GenericEntity]
var friends: Array[int]
var player_pid: int

func _ready() -> void:
	position = get_global_mouse_position()
	
	for player in players:
		add_inspect_option(player.data)
	for entity in entities:
		if entity.interactable:
			add_interact_option(entity)
	var walk_button = Button.new()
	walk_button.text = "Walk here"
	walk_button.pressed.connect(walk_to_pos.emit.bind(related_pos))
	
	walk_button.pressed.connect(self.queue_free)
	vbox.add_child(walk_button)

func add_inspect_option(data: PlayerContainer) -> void:
	var interact_string := "Inspect player \"%s\"" % data.get_name()
	var new_button = Button.new()
	new_button.text = interact_string
	
	var pid := data.get_pid()
	var show_invite_btn := pid != player_pid and not friends.has(pid)
	var show_dm_button := pid != player_pid
	new_button.pressed.connect(inspect_player.emit.bind(data, show_invite_btn, show_dm_button))
	new_button.pressed.connect(self.queue_free)
	vbox.add_child(new_button)

func add_interact_option(entity: GenericEntity) -> void:
	var interact_string: String
	if not entity.interactable_string.is_empty():
		interact_string = entity.interactable_string
	else:
		interact_string = "Interact with"
	interact_string = interact_string + " " + entity.visible_name
	var new_button = Button.new()
	new_button.text = interact_string
	new_button.pressed.connect(interact_with_entity.emit.bind(entity))
	new_button.pressed.connect(self.queue_free)
	vbox.add_child(new_button)
