class_name InventoryItem
extends TextureRect

@onready var item_rect: TextureRect = get_node("Item")
@onready var item_count: Label = get_node("ItemCount")
@onready var highlight: ColorRect = get_node("Highlight")

# Sent back in signals
var index: int
var item_resource: ItemResource

signal slot_clicked(index: int)

func _gui_input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		if event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
			emit_index()

func emit_index() -> void:
	slot_clicked.emit(index)

func set_index(i: int) -> void:
	index = i

func reset_data() -> void:
	item_count.text = ""
	item_rect.texture = null
	item_rect.visible = true
	tooltip_text = ""

func set_item(item: ItemResource) -> void:
	reset_data()
	item_resource = item
	if item == null:
		return
	load_item_icon(item.id_string)
	if item.count > 1:
		item_count.text = str(item.count)
	tooltip_text = item.description

func load_item_icon(id_string: String) -> void:
	var texture: Texture2D = load("res://graphics/icons/%s.png" % id_string)
	
	if texture == null:
		print("Couldn't find item icon for ", id_string)
	else:
		item_rect.texture = texture

func set_icon_visible(vis: bool) -> void:
	item_rect.visible = vis

func get_icon_texture() -> Texture2D:
	return item_rect.texture

func _on_hover() -> void:
	highlight.visible = true

func _on_unhover() -> void:
	highlight.visible = false
