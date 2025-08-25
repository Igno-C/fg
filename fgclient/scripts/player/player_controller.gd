class_name PlayerController

extends Node

var player: PlayerEntity
var camera: Camera2D
var map: BaseMap

var predictions = []

var go_to_target: bool = false
var target: Vector2i = Vector2i.ZERO

#const MLEFT: int = 0; const MRIGHT: int = 1; const MUP: int = 2; const MDOWN: int = 3
# speed 3 is normal, 2 is sprint, 1 is fastest
var next_speed: int = 3
var previous_speed: int = 3
var ticks_since_move: int = 100

var pis = PlayerInputState.new()

signal send_move(x: int, y: int, speed: int)
signal send_event(event: GenericEvent)
signal set_debug_label(text: String)

func _process(_delta:) -> void:
	if not go_to_target:
		target = player.pos
	
	if pis.up_pressed: target.y = player.pos.y - 1
	elif pis.up_just_released: target.y = player.pos.y
	if pis.down_pressed: target.y = player.pos.y + 1
	elif pis.down_just_released: target.y = player.pos.y
	if pis.left_pressed: target.x = player.pos.x - 1
	elif pis.left_just_released: target.x = player.pos.x
	if pis.right_pressed: target.x = player.pos.x + 1
	elif pis.right_just_released: target.x = player.pos.x
	
	if pis.sprint_pressed: next_speed = 2
	else: next_speed = 3
	
	go_to_target = target != player.pos
	pis.tick()

func get_mouse_ipos() -> Vector2i:
	var pos: Vector2 = camera.get_local_mouse_position() + camera.global_position
	pos /= 50.0
	
	var ipos: Vector2i = Vector2i(pos.floor())
	return ipos

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		if event.button_index == MOUSE_BUTTON_LEFT:
			var ipos := get_mouse_ipos()
			target = ipos
			go_to_target = true
		elif event.button_index == MOUSE_BUTTON_RIGHT:
			var ipos := get_mouse_ipos()
			print(ipos)
			send_event.emit(GenericEvent.interaction(ipos.x, ipos.y))
	else:
		pis.take_event(event)

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
			print("didn't send ", nextpos, ", speed: ", next_speed, " (predicted collision)")

func receive_move(x: int, y: int, speed: int) -> void:
	var pred = predictions.pop_front()
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
	
	var to_pos := Vector2i(x, y)
	player.move(to_pos, 0)
	target = to_pos
	ticks_since_move = 0
	previous_speed = 0

func delta_from_target() -> Vector2i:
	var delta: Vector2i = target - player.pos
	delta = delta.clampi(-1, 1)
	
	return player.pos + delta
