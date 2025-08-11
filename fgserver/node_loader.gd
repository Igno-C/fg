extends Node

@onready var server_list: ServerList = get_node("/root/ServerList")


func _ready() -> void:
	var root: Node = get_tree().root
	
	var manager_node := GameManager.from_config()
	manager_node.ready.connect(server_list._on_manager_ready)
	manager_node.name = "ManagerNode"
	root.add_child.call_deferred(manager_node)
	
	var config := ConfigFile.new()
	config.load("res://serverconfig.cfg")
	var max_players: int = config.get_value("Server", "max_players")
	var port: int = config.get_value("Server", "port")
	var server_node := Server.from_config(port, max_players)
	server_node.ready.connect(server_list._on_server_ready)
	server_node.name = "ServerNode"
	root.add_child.call_deferred(server_node)
	
	var queue_initializer: Node = get_node("/root/QueueNode")
	queue_initializer.queue_free()
