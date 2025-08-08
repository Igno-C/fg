class_name PlayerEntity

extends GenericEntity

@onready var name_label: Label = get_node("NameLabel")

var data: PlayerContainer

func receive_data(newdata: PlayerContainer) -> void:
	data = newdata
	name_label.text = data.get_name()
