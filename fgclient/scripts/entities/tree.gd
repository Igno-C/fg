extends Sprite2D

var is_cut: bool = false

func receive_data(data: Dictionary) -> void:
	var kind = data.get("kind")
	var is_cut: bool = data.get("cut")
	var tree_sprite: Texture2D
	if is_cut:
		tree_sprite = load("res://graphics/entities/stump.png")
	elif kind == "oak":
		tree_sprite = load("res://graphics/entities/oak.png")
	elif kind == "fir":
		tree_sprite = load("res://graphics/entities/fir.png")
	if tree_sprite != null:
		texture = tree_sprite
