extends Control

var entry_scene: PackedScene = preload("res://scenes/menu/login/ServerListEntry.tscn")
@onready var vbox: VBoxContainer = get_node("ServerList/Margin/VBoxContainer/ScrollContainer/VBox")


signal pressed(server_name: String)
signal refresh_request


func set_enabled(enabled: bool) -> void:
	for child in vbox.get_children():
		child.set_enabled(enabled)

func show_box(show: bool) -> void:
	visible = show
	if show:
		clear_list()

func clear_list() -> void:
	for child in vbox.get_children():
		child.queue_free()

func populate(servers: Array[Dictionary]) -> void:
	clear_list()
	
	# Create an entry for each server
	for server in servers:
		var entry = entry_scene.instantiate()
		
		entry.with_server(server["name"], server["load"])
		entry.pressed.connect(_on_entry_pressed)
		
		vbox.add_child(entry)

func _on_refresh_pressed() -> void:
	refresh_request.emit()

func _on_entry_pressed(server_name: String) -> void:
	set_enabled(false)
	pressed.emit(server_name)
