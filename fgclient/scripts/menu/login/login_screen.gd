extends Control

@onready var gateway: GatewayServer = get_node("/root/GatewayServer")

@onready var login_box: Control = get_node("LoginBox")
@onready var creation_box: Control = get_node("CreationBox")
@onready var server_list: Control = get_node("ServerList")
@onready var loading_box: Control = get_node("LoadingBox")

signal login_success(pid: int, ip: String, port: int, token: String)


func _ready() -> void:
	gateway.timeout.connect(_on_login_timeout)
	gateway.other_error.connect(_on_login_other_error)
	gateway.unreachable.connect(_on_gateway_unreachable)
	
	login_box.pressed.connect(_on_login_press)
	login_box.creation_pressed.connect(_on_login_creation_press)
	gateway.success.connect(_on_login_success)
	gateway.invalid.connect(_on_login_invalid)
	
	creation_box.pressed.connect(_on_creation_press)
	creation_box.go_back_pressed.connect(_on_creation_go_back_press)
	gateway.creation_status.connect(_on_creation_status)
	
	server_list.pressed.connect(_on_server_selected)
	gateway.got_server_list.connect(_on_got_server_list)
	gateway.joined_server.connect(_on_joined_server)
	
	open_login()
	#var sl: Array[Dictionary] = [{"name": "a", "load": 1}, {"name": "bbbbbbbb", "load": 4}]
	#sl += sl; sl += sl; sl += sl; sl += sl; sl += sl; 
	#print(sl)
	#server_list.populate(sl)

func _on_joined_server(pid: int, ip: String, port: int, token: String) -> void:
	for child in get_children():
		child.visible = false
	loading_box.visible = true
	login_success.emit(pid, ip, port, token)

func open_login() -> void:
	login_box.show_box(true)
	creation_box.show_box(false)
	server_list.show_box(false)

func open_creation() -> void:
	login_box.show_box(false)
	creation_box.show_box(true)
	server_list.show_box(false)

func open_server_list() -> void:
	login_box.show_box(false)
	creation_box.show_box(false)
	server_list.show_box(true)

func validate_lengths(username: String, password: String) -> bool:
	if username.length() < 5 or username.length() > 20:
		return false
	if password.length() < 8 or password.length() > 40:
		return false
	return true


# Generic errors
func _on_login_timeout() -> void:
	var e: String = "Error: Connection to gateway timed out"
	open_login()
	login_box.set_err(e)

func _on_login_other_error(err: int) -> void:
	var e: String = "Error: Connection to gateway failed with code " + str(err)
	open_login()
	login_box.set_err(e)

func _on_gateway_unreachable() -> void:
	var e: String = "Error: Could not reach gateway"
	open_login()
	login_box.set_err(e)


# Login section
func _on_login_press(username: String, password: String) -> void:
	if validate_lengths(username, password):
		gateway.send_credentials(username, password)
		login_box.set_enabled(false)
	else:
		login_box.set_err("Usernames must be between 5-20 characters\nPasswords between 8-40")

func _on_login_creation_press() -> void:
	open_creation()

func _on_login_success() -> void:
	open_server_list()
	gateway.send_server_list_request()

func _on_login_invalid() -> void:
	var e: String = "Error: Username or password invalid"
	
	login_box.set_err(e)
	login_box.set_enabled(true)


# Creation section
func _on_creation_press(username: String, password: String, password2: String) -> void:
	if validate_lengths(username, password):
		if password == password2:
			gateway.send_creation(username, password)
			creation_box.set_enabled(false)
			#set_input_enable(false)
		else:
			creation_box.set_err("Passwords don't match")
	else:
		creation_box.set_err("Username or password is too short")

func _on_creation_go_back_press() -> void:
	open_login()

# Response from the gateway server
func _on_creation_status(valid: bool) -> void:
	if valid:
		open_login()
		login_box.set_err("Account created!", false)
	else:
		creation_box.set_enabled(true)
		creation_box.set_err("Username or email already in use")


# Server list section
func _on_got_server_list(list: Array[Dictionary]) -> void:
	server_list.populate(list)

func _on_server_selected(name: String) -> void:
	gateway.send_chosen_server(name)
