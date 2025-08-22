@tool
extends BaseMap

func _notification(what: int) -> void:
	if what == Node.NOTIFICATION_EDITOR_POST_SAVE:
		var this_packedscene: PackedScene = load(scene_file_path)
		
		var scene_name: String = scene_file_path.rsplit("/", true, 1)[1].trim_suffix(".tscn")
		print("Saving scene \"%s\"" % scene_name)
		
		var this_scene: BaseMap = this_packedscene.instantiate()
		var entities: Node = this_scene.get_node("Entities")
		this_scene.remove_child(entities)
		this_scene.set_script(null)
		
		# Needed so that PackedScene.pack() saves it correctly
		# Otherwise, the BaseMap node is still the owner
		for child in entities.get_children():
			child.owner = entities
		
		var client_scene := PackedScene.new()
		var server_scene := PackedScene.new()
		client_scene.pack(this_scene)
		server_scene.pack(entities)
		#client_scene.instantiate().print_tree_pretty()
		#server_scene.instantiate().print_tree_pretty()
		
		var client_path := "res://exports/client/%s.tscn" % scene_name
		var server_path := "res://exports/server/%s.tscn" % scene_name
		var server_collision_path := "res://exports/server/%s.col" % scene_name

		ResourceSaver.save(client_scene, client_path)
		ResourceSaver.save(server_scene, server_path)
		var collision_data = get_collision_bytes()
		var col_file := FileAccess.open(server_collision_path, FileAccess.WRITE)
		col_file.store_buffer(collision_data)
		col_file.close()
		
		entities.queue_free()
		this_scene.queue_free()
