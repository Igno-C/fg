extends Sprite2D

func receive_data(data: Dictionary) -> void:
	var kind = data.get("kind")
	var tree_sprite: Texture2D
	if kind == "oak":
		tree_sprite = load("res://graphics/entities/oak.png")
	elif kind == "fir":
		tree_sprite = load("res://graphics/entities/fir.png")
	if tree_sprite != null:
		texture = tree_sprite
