extends Control

@onready var main_inv: ColorRect = get_node("MainInv")
@onready var item_grid: GridContainer = get_node("%Grid")
@onready var equipped_item: InventoryItem = get_node("%EquippedSlot")
@onready var bag_icon: Button = get_node("BagButton")
@onready var gold_count: Label = get_node("%GoldCount")

var item_nodes: Array[Control] = []
signal slot_clicked(index: int)

func _ready() -> void:
	bag_icon.pressed.connect(toggle)
	var item_scene: PackedScene = load("res://scenes/menu/game/InventoryItem.tscn")
	for i in range(40):
		var item: Control = item_scene.instantiate()
		item.set_index(i)
		item_grid.add_child(item)
		item_nodes.push_back(item)
		item.slot_clicked.connect(slot_clicked.emit)

func populate(data: PlayerContainer) -> void:
	var items := data.get_items()
	var i := 0
	for item in items:
		var item_node = item_nodes[i]
		item_node.set_item(item)
		i += 1
	var eq_item := data.get_equipped_item()
	equipped_item.set_item(eq_item)
	gold_count.text = str(data.get_gold())

func toggle() -> void:
	main_inv.visible = not main_inv.visible
	if main_inv.visible:
		bag_icon.position.y -= 330
	else:
		bag_icon.position.y += 330
