extends Control

func populate(data: PlayerContainer) -> void:
	visible = true

func _on_close_button_pressed() -> void:
	visible = false
