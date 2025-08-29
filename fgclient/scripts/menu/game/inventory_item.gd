class_name InventoryItem
extends TextureRect

@onready var item: TextureRect = get_node("Item")
@onready var item_count: Label = get_node("ItemCount")
@onready var highlight: ColorRect = get_node("Highlight")

# Sent back in signals
var index: int

signal slot_clicked(index: int)

func set_index(i: int) -> void:
	index = i

func reset_data() -> void:
	item_count.text = ""
	item.texture = null

func set_item(item: ItemResource) -> void:
	reset_data()
	if item == null:
		return
	load_item_icon(item.id_string)
	if item.count > 1:
		item_count.text = str(item.count)

func load_item_icon(id_string: String) -> void:
	var texture: Texture2D = load("res://graphics/icons/%s.png" % id_string)
	
	if texture == null:
		print("Couldn't find item icon for ", id_string)
	else:
		item.texture = texture

func _on_hover() -> void:
	highlight.visible = true

func _on_unhover() -> void:
	highlight.visible = false
