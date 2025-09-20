class_name GatewayWaiter

const TIMEOUT: float = 4.
const AUTHENTICATED_TIMEOUT: float = 35.

var authenticated: bool = false
var pid: int = -1

var time: float = 0.
var got_request: bool = false

var username: String

# Returns true past timeout
func tick_timeout(delta: float) -> bool:
	time += delta
	if authenticated:
		if time > AUTHENTICATED_TIMEOUT:
			return true
	else:
		if time > TIMEOUT:
			return true
	return false

func authenticate(p: int) -> void:
	authenticated = true
	pid = p
	time = 0.

# Returns true if this is the first request received, otherwise false
func received_request():
	if got_request:
		return false
	time = 0.
	got_request = true
	return true
