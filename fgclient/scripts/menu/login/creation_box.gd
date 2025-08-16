extends Control

@onready var email: LineEdit = get_node("CreationBox/Margin/VBox/EmailEdit")
@onready var username: LineEdit = get_node("CreationBox/Margin/VBox/UsernameEdit")
@onready var password: LineEdit = get_node("CreationBox/Margin/VBox/PasswordEdit")
@onready var password2: LineEdit = get_node("CreationBox/Margin/VBox/PasswordEdit2")
@onready var button: Button = get_node("CreationBox/Margin/VBox/CreationButton")
@onready var go_back_button: Button = get_node("CreationBox/GoBackButton")
@onready var err_label: Label = get_node("CreationBox/Margin/VBox/ErrorLabel")

signal pressed(username: String, password: String, password2: String)
signal go_back_pressed


func set_err(text: String, is_red: bool = true) -> void:
	if is_red:
		err_label.add_theme_color_override("font_color", Color.RED)
	else:
		err_label.add_theme_color_override("font_color", Color.GREEN)
	err_label.text = text

func set_enabled(enabled: bool) -> void:
	email.editable = enabled
	username.editable = enabled
	password.editable = enabled
	password2.editable = enabled
	button.disabled = not enabled
	go_back_button.disabled = not enabled

func show_box(show: bool) -> void:
	visible = show
	if show:
		email.text = ""
		username.text = ""
		password.text = ""
		password2.text = ""
		err_label.text = ""
		set_enabled(true)

func _on_password_submitted(text: String) -> void:
	err_label.text = ""
	pressed.emit(username.text, password.text, text)

func _on_button_pressed() -> void:
	err_label.text = ""
	pressed.emit(username.text, password.text, password2.text)

func _on_go_back_pressed() -> void:
	go_back_pressed.emit()
