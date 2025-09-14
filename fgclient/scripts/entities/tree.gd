extends GenericEntity

@onready var sprite: Sprite2D = $Tree

var is_cut: bool = false

func _ready() -> void:
	interactable_string = "Cut down"
	visible_name = "Tree"

func receive_data(data: Dictionary) -> void:
	var kind = data.get("kind")
	var is_cut: bool = data.get("cut")
	var tree_sprite: Texture2D
	if is_cut:
		tree_sprite = load("res://graphics/entities/stump.png")
	elif kind == "oak":
		tree_sprite = load("res://graphics/entities/oak.png")
		visible_name = "Oak Tree"
	elif kind == "fir":
		tree_sprite = load("res://graphics/entities/fir.png")
		visible_name = "Fir Tree"
	if tree_sprite != null:
		sprite.texture = tree_sprite
