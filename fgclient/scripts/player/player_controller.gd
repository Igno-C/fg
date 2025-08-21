class_name PlayerController

extends Node

var player: PlayerEntity
var camera: Camera2D
var map: BaseMap

var predictions = []

var go_to_target: bool = false
var target: Vector2i = Vector2i.ZERO

const MLEFT: int = 0; const MRIGHT: int = 1; const MUP: int = 2; const MDOWN: int = 3
#var pressed_moves: Array[int] = []
#var unpressed_moves: Array[int] = [] # Used to buffer inputs to never miss them
# speed 3 is normal, 2 is sprint, 1 is fastest
var next_speed: int = 3
var previous_speed: int = 3
var ticks_since_move: int = 100


signal send_move(x: int, y: int, speed: int)
signal send_event(event: GenericEvent)
signal set_debug_label(text: String)

func _process(_delta:) -> void:
	if not go_to_target:
		target = player.pos
	
	if Input.is_action_pressed("Up"): target.y = player.pos.y - 1
	elif Input.is_action_pressed("Down"): target.y = player.pos.y + 1
	elif Input.is_action_just_released("Up"): target.y = player.pos.y
	elif Input.is_action_just_released("Down"): target.y = player.pos.y
	
	if Input.is_action_pressed("Right"): target.x = player.pos.x + 1
	elif Input.is_action_pressed("Left"): target.x = player.pos.x - 1
	elif Input.is_action_just_released("Right"): target.x = player.pos.x
	elif Input.is_action_just_released("Left"): target.x = player.pos.x
	
	if Input.is_action_just_released("Sprint"): next_speed = 3 # Note, bigger speed means slower
	if Input.is_action_just_pressed("Sprint"): next_speed = 2
	
	if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
		var ipos := get_mouse_ipos()
		target = ipos
	if Input.is_mouse_button_pressed(MOUSE_BUTTON_RIGHT):
		var ipos := get_mouse_ipos()
		print(ipos)
		#var target := player.pos + player.get_dir_vec()
		send_event.emit(GenericEvent.interaction(ipos.x, ipos.y))
	
	if target != player.pos:
		go_to_target = true
	else:
		go_to_target = false

func get_mouse_ipos() -> Vector2i:
	var pos: Vector2 = camera.get_local_mouse_position() + camera.global_position
	pos /= 50.0
	
	var ipos: Vector2i = Vector2i(pos.floor())
	return ipos

#func _input(event: InputEvent) -> void:
	#if player == null:
		#return
	#if event is InputEventMouseButton:
		#if event.button_index == 1:
			#var pos: Vector2 = camera.get_local_mouse_position() + camera.global_position
			#pos /= 50.0
			#
			#var ipos: Vector2i = Vector2i(pos.floor())
			#print(ipos)

func set_player(p: PlayerEntity) -> void:
	player = p

func set_map(m: BaseMap) -> void:
	map = m

func on_tick() -> void:
	if player == null:
		return
	
	ticks_since_move += 1
	if ticks_since_move >= previous_speed and go_to_target:
		#var inp = delta_from_direction()
		#if inp != Vector2i.ZERO:
		var nextpos := delta_from_target()
		# collision checking
		if !map.get_at(nextpos.x, nextpos.y):
			send_move.emit(nextpos.x, nextpos.y, next_speed) # To send to server
			predictions.push_back([nextpos, next_speed])
			if predictions.size() > 5:
				var s = "Worryingly many predictions: %s" % predictions.size()
				print(s)
				set_debug_label.emit(s)
			else:
				set_debug_label.emit("")
			player.move(nextpos, next_speed) # Visual client-side movement
			ticks_since_move = 0; previous_speed = next_speed
			print("sent ", nextpos, ", speed: ", next_speed)
		else:
			go_to_target = false
			target = player.pos
			print("didn't send ", nextpos, ", speed: ", next_speed, " (predicted collision)")
	
	#for move in unpressed_moves:
		#pressed_moves.erase(move)
	#unpressed_moves.clear()

func receive_move(x: int, y: int, speed: int) -> void:
	var pred = predictions.pop_front()
	set_debug_label.emit(str(predictions))
	if pred != null:
		var predpos = pred[0]; var predspeed = pred[1]
		if x == predpos.x and y == predpos.y and speed == predspeed:
			return # The prediction was correct
		else:
			# Try the next prediction just in case this was caused by packet loss
			pred = predictions.pop_front()
			if pred != null:
				predpos = pred[0]; predspeed = pred[1]
				if x == predpos.x and y == predpos.y and speed == predspeed:
					return # The next prediction was correct
				print("Prediction mismatch, received: (", x, ", ", y, "), predicted: (", predpos, ")")
				predictions.clear()
	player.move(Vector2i(x, y), 0); ticks_since_move = 0; previous_speed = 0

func delta_from_target() -> Vector2i:
	var delta: Vector2i = target - player.pos
	delta = delta.clampi(-1, 1)
	
	return player.pos + delta

#func delta_from_direction() -> Vector2i:
	#if pressed_moves.is_empty():
		#return Vector2i.ZERO
	#match pressed_moves[-1]:
		#MUP:
			#return Vector2i.UP
		#MDOWN:
			#return Vector2i.DOWN
		#MLEFT:
			#return Vector2i.LEFT
		#MRIGHT:
			#return Vector2i.RIGHT
		#_:
			#return Vector2i.ZERO
