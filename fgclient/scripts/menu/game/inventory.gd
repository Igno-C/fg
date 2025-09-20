extends Control

@onready var main_inv: ColorRect = get_node("MainInv")
@onready var item_grid: GridContainer = get_node("%Grid")
@onready var equipped_item: InventoryItem = get_node("%EquippedSlot")
@onready var bag_icon: Button = get_node("BagButton")
@onready var gold_count: Label = get_node("%GoldCount")
@onready var held_item_node: TextureRect = get_node("HeldItem")

var item_nodes: Array[InventoryItem] = []
#signal slot_clicked(index: int)

## -2 means holding equipped item
var held_item_index: int = -1

func _on_slot_clicked(index: int) -> void:
	if held_item_index == -1:
		start_holding_item(index)
	elif held_item_index == -2:
		var event = GenericEvent.equip_item(index)
		ServerNode.send_event(event)
		swap_eq_and_index(index)
		stop_holding_item()
	else:
		if held_item_index != index:
			var event = GenericEvent.swap_items(held_item_index, index)
			ServerNode.send_event(event)
		swap_held_and_index(index)
		stop_holding_item()

func _on_equipped_clicked(_this_is_minustwo: int) -> void:
	if held_item_index == -1:
		start_holding_item(-2)
	elif held_item_index == -2:
		stop_holding_item()
	else:
		var event = GenericEvent.equip_item(held_item_index)
		ServerNode.send_event(event)
		swap_eq_and_index(held_item_index)
		stop_holding_item()

func swap_eq_and_index(index: int) -> void:
	var eq_item = equipped_item.item_resource
	var held_item = item_nodes[index].item_resource
	item_nodes[index].set_item(eq_item)
	equipped_item.set_item(held_item)

func swap_held_and_index(index: int) -> void:
	var other_item = item_nodes[index].item_resource
	var held_item = item_nodes[held_item_index].item_resource
	item_nodes[held_item_index].set_item(other_item)
	item_nodes[index].set_item(held_item)

func start_holding_item(index: int) -> void:
	if held_item_index == -1:
		if index == -2:
			held_item_index = -2
			equipped_item.set_icon_visible(false)
			held_item_node.texture = equipped_item.get_icon_texture()
			held_item_node.visible = true
		elif index != -1:
			held_item_index = index
			item_nodes[held_item_index].set_icon_visible(false)
			held_item_node.texture = item_nodes[held_item_index].get_icon_texture()
			held_item_node.visible = true

func stop_holding_item() -> void:
	if held_item_index == -2:
		equipped_item.set_icon_visible(true)
		held_item_index = -1
		held_item_node.visible = false
	elif held_item_index != -1:
		item_nodes[held_item_index].set_icon_visible(true)
		held_item_index = -1
		held_item_node.visible = false

func _ready() -> void:
	bag_icon.pressed.connect(toggle)
	var item_scene: PackedScene = load("res://scenes/menu/game/InventoryItem.tscn")
	for i in range(40):
		var item: InventoryItem = item_scene.instantiate()
		item.set_index(i)
		item_grid.add_child(item)
		item_nodes.push_back(item)
		item.slot_clicked.connect(_on_slot_clicked)
	equipped_item.slot_clicked.connect(_on_equipped_clicked)
	equipped_item.set_index(-2)

func _process(delta: float) -> void:
	if held_item_node.visible:
		held_item_node.position = get_global_mouse_position()

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
	held_item_node.visible = main_inv.visible
	held_item_node.texture = null
	if held_item_index != -1:
		item_nodes[held_item_index].set_icon_visible(true)
		held_item_index = -1
	if main_inv.visible:
		bag_icon.position.y -= 330
	else:
		bag_icon.position.y += 330
