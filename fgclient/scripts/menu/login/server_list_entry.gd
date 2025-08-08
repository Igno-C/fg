extends PanelContainer

@onready var icon: TextureRect = get_node("MarginContainer/GridContainer/TextureRect")
@onready var label: Label = get_node("MarginContainer/GridContainer/Label")
@onready var button: Button = get_node("MarginContainer/GridContainer/Button")
var server_name: String
var server_load: int

var load_icons: Array[Texture2D] = [
	preload("res://graphics/menu/load1.png"),
	preload("res://graphics/menu/load2.png"),
	preload("res://graphics/menu/load3.png"),
	preload("res://graphics/menu/load4.png"),
]

signal pressed(name: String)


func set_enabled(enabled: bool) -> void:
	button.disabled = not enabled

func _ready() -> void:
	label.text = server_name
	
	var idx = clampi(server_load - 1, 0, load_icons.size() - 1)
	icon.texture = load_icons[idx]
	button.icon = load_icons[idx]

func with_server(name: String, load: int) -> void:
	server_name = name
	server_load = load

func _on_button_pressed() -> void:
	pressed.emit(server_name)
