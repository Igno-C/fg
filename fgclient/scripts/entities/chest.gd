extends Sprite2D

var is_open: bool = false

func receive_data(data: Dictionary) -> void:
	is_open = data["open"]
	
	if is_open:
		texture = load("res://graphics/entities/chest_open.png")
	else:
		texture = load("res://graphics/entities/chest.png")
