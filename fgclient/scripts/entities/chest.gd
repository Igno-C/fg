extends GenericEntity

@onready var sprite: Sprite2D = $Chest

var is_open: bool = false

func _ready() -> void:
	visible_name = "Chest"
	interactable_string = "Open"

func receive_data(data: Dictionary) -> void:
	is_open = data["open"]
	
	if is_open:
		sprite.texture = load("res://graphics/entities/chest_open.png")
	else:
		sprite.texture = load("res://graphics/entities/chest.png")
