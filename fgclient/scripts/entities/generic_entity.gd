class_name GenericEntity

extends GenericMovable

var data: Dictionary = {}
var data_version: int = -1

func receive_data(new_data: Dictionary):
	data = new_data
	print()
