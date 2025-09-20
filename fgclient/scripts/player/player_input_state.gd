class_name PlayerInputState

var left_pressed: bool = false
var left_just_released: bool = false
var right_pressed: bool = false
var right_just_released: bool = false
var up_pressed: bool = false
var up_just_released: bool = false
var down_pressed: bool = false
var down_just_released: bool = false

var sprint_pressed: bool = false

## To be run at the end of _process() to reset just pressed and the like
func tick() -> void:
	left_just_released = false
	right_just_released = false
	up_just_released = false
	down_just_released = false

func take_event(event: InputEvent) -> void:
	if event.is_action_pressed("Left"): left_pressed = true
	elif event.is_action_released("Left"): left_pressed = false; left_just_released = true
	elif event.is_action_pressed("Right"): right_pressed = true
	elif event.is_action_released("Right"): right_pressed = false; right_just_released = true
	elif event.is_action_pressed("Up"): up_pressed = true
	elif event.is_action_released("Up"): up_pressed = false; up_just_released = true
	elif event.is_action_pressed("Down"): down_pressed = true
	elif event.is_action_released("Down"): down_pressed = false; down_just_released = true
	elif event.is_action_pressed("Sprint"): sprint_pressed = true
	elif event.is_action_released("Sprint"): sprint_pressed = false
