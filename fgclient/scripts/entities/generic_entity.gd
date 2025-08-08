class_name GenericEntity

extends Node2D

# Size in pixels of a tile
const posmult: float = 50.0

# Logical discrete position
var pos: Vector2i
var direction: Dir = Dir.DOWN
enum Dir {UP, DOWN, LEFT, RIGHT}

# Used for interpolation
var target: Vector2
var speed: int
var currently_moving: bool = false

# 'by' is the distance that should be travelled
func real_lerp(to: Vector2, from: Vector2, by: float) -> Vector2:
	var delta: Vector2 = to - from
	var movement: Vector2 = delta.normalized() * by
	if movement.length_squared() > delta.length_squared():
		movement = delta
	return from + movement

# newx: int, newy: int
func move(newpos: Vector2i, newspeed: int) -> void:
	var delta = newpos - pos
	if delta.x == 1: direction = Dir.RIGHT
	elif delta.x == -1: direction = Dir.LEFT
	elif delta.y == 1: direction = Dir.DOWN
	elif delta.y == -1: direction = Dir.UP
	currently_moving = true
	pos = newpos
	target = pos * posmult
	#target.x = x*posmult
	#target.y = y*posmult
	speed = newspeed
	#print("Moving entity to newx: ", newx, ", newy: ", newy, ", speed: ", speed)

func set_direction(dir: Dir) -> void:
	# Setting stuff like sprite animation should happen here
	#
	#
	direction = dir

func get_dir_vec() -> Vector2i:
	match direction:
		Dir.UP:
			return Vector2i.UP
		Dir.DOWN:
			return Vector2i.DOWN
		Dir.LEFT:
			return Vector2i.LEFT
		Dir.RIGHT:
			return Vector2i.RIGHT
		_:
			return Vector2i.ZERO

func _process(delta: float) -> void:
	if currently_moving:
		if speed == 0:
			position = target
			currently_moving = false
		else:
			position = real_lerp(target, position, posmult / speed * delta * 10)
			if position == target:
				currently_moving = false
