extends Node

@onready var benchmark_node: BenchmarkNode = $BenchmarkNode

func _ready() -> void:
	benchmark_node.do_perf_bench()
	
	var cmap: TileMapLayer = load("res://cmap_scene.tscn").instantiate()
	
	print("This many used cells ", cmap.get_used_cells().size())
