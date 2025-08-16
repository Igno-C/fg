class_name ServerDirections

var ip: String
var port: int
var token: String

func _init(i: String, p: int, t: String) -> void:
	ip = i; port = p; token = t

func _to_string() -> String:
	return "%s:%s, with token %s" % [ip, port, token]
