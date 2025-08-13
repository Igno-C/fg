class_name PlayerController

extends Node

var player: PlayerEntity
var camera: Camera2D
var map: BaseMap

var predictions = []

const MLEFT: int = 0; const MRIGHT: int = 1; const MUP: int = 2; const MDOWN: int = 3
var pressed_moves: Array[int] = []
var unpressed_moves: Array[int] = [] # Used to buffer inputs to never miss them
# speed 3 is normal, 2 is sprint, 1 is fastest
var next_speed: int = 3
var previous_speed: int = 3
var ticks_since_move: int = 100


signal send_move(x: int, y: int, speed: int)
signal send_event(event: GenericEvent)
signal set_debug_label(text: String)

func _process(_delta:) -> void:
	if Input.is_action_just_released("Up"): unpressed_moves.push_back(MUP)
	if Input.is_action_just_released("Down"): unpressed_moves.push_back(MDOWN)
	if Input.is_action_just_released("Right"): unpressed_moves.push_back(MRIGHT)
	if Input.is_action_just_released("Left"): unpressed_moves.push_back(MLEFT)
	if Input.is_action_just_released("Sprint"): next_speed = 3
	
	if Input.is_action_just_pressed("Right"): pressed_moves.push_back(MRIGHT)
	if Input.is_action_just_pressed("Left"): pressed_moves.push_back(MLEFT)
	if Input.is_action_just_pressed("Up"): pressed_moves.push_back(MUP)
	if Input.is_action_just_pressed("Down"): pressed_moves.push_back(MDOWN)
	if Input.is_action_just_pressed("Sprint"): next_speed = 2
	
	if Input.is_action_just_pressed("Interact"):
		var target := player.pos + player.get_dir_vec()
		send_event.emit(GenericEvent.interaction(target.x, target.y))

func _input(event: InputEvent) -> void:
	if player == null:
		return
	if event is InputEventMouseButton:
		if event.button_index == 1:
			var pos: Vector2 = camera.get_local_mouse_position() + camera.global_position
			pos /= 50.0
			
			var ipos: Vector2i = Vector2i(pos.floor())
			print(ipos)

func set_player(p: PlayerEntity) -> void:
	player = p

func set_map(m: BaseMap) -> void:
	map = m

func on_tick() -> void:
	if player == null:
		return
	
	ticks_since_move += 1
	if ticks_since_move >= previous_speed:
		var inp = delta_from_direction()
		if inp != Vector2i.ZERO:
			var nextpos = player.pos + inp
			# collision checking
			if !map.get_at(nextpos.x, nextpos.y):
				send_move.emit(nextpos.x, nextpos.y, next_speed) # To send to server
				predictions.push_back([nextpos, next_speed])
				if predictions.size() > 5:
					print("Worryingly many predictions: ", predictions.size())
				player.move(nextpos, next_speed) # Visual client-side movement
				ticks_since_move = 0; previous_speed = next_speed
				print("sent ", nextpos, ", speed: ", next_speed)
			else:
				print("didn't send ", nextpos, ", speed: ", next_speed, " (predicted collision)")
	
	for move in unpressed_moves:
		pressed_moves.erase(move)
	unpressed_moves.clear()

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

func delta_from_direction() -> Vector2i:
	if pressed_moves.is_empty():
		return Vector2i.ZERO
	match pressed_moves[-1]:
		MUP:
			return Vector2i.UP
		MDOWN:
			return Vector2i.DOWN
		MLEFT:
			return Vector2i.LEFT
		MRIGHT:
			return Vector2i.RIGHT
		_:
			return Vector2i.ZERO
