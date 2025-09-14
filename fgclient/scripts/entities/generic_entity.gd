class_name GenericEntity

extends GenericMovable

## The name that will actually be shown to the player
var visible_name: String = "Entity"
var entity_id: int = -1
var data: Dictionary = {}
var data_version: int = -1
#var scene: Node = null
#var scene_name: String = ""

var interactable: bool = false
## Shown in context menu instead of "Interact with"
var interactable_string: String = ""
var walkable: bool = false

#func _init(_data: Dictionary) -> void:
	#data = _data

#func load_scene(new_scene_name: String) -> void:
	#if scene_name == new_scene_name:
		#return
	#
	#if scene != null:
		#scene.queue_free()
		#scene = null
	#
	#scene_name = new_scene_name
	#
	#if not scene_name.is_empty():
		#var 
		#scene = packed_scene.instantiate()
		#add_child(scene)

func receive_data(new_data: Dictionary) -> void:
	data = new_data
