extends Control

@onready var username: LineEdit = get_node("LoginBox/Margin/VBox/UsernameEdit")
@onready var password: LineEdit = get_node("LoginBox/Margin/VBox/PasswordEdit")
@onready var button: Button = get_node("LoginBox/Margin/VBox/LoginButton")
@onready var creation_button: LinkButton = get_node("LoginBox/Margin/VBox/CreationButton")
@onready var err_label: Label = get_node("LoginBox/Margin/VBox/ErrorLabel")

signal pressed(username: String, password: String)
signal creation_pressed


func set_err(text: String, is_red: bool = true) -> void:
	if is_red:
		err_label.add_theme_color_override("font_color", Color.RED)
	else:
		err_label.add_theme_color_override("font_color", Color.GREEN)
	err_label.text = text

func set_enabled(enabled: bool) -> void:
	username.editable = enabled
	password.editable = enabled
	button.disabled = not enabled
	creation_button.disabled = not enabled

func show_box(show: bool) -> void:
	visible = show
	if show:
		username.text = ""
		password.text = ""
		err_label.text = ""
		set_enabled(true)

func _on_button_pressed() -> void:
	err_label.text = ""
	pressed.emit(username.text, password.text)

func _on_creation_button_pressed() -> void:
	creation_pressed.emit()
