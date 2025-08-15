class_name PlayerEntity

extends GenericEntity

@onready var sprite: Sprite2D = get_node("Sprite2D")
@onready var name_label: Label = get_node("NameLabel")

var data: PlayerContainer

func receive_data(newdata: PlayerContainer) -> void:
	data = newdata
	name_label.text = data.get_name()
	sprite.visible = true
