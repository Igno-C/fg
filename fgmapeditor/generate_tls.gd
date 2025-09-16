@tool
extends EditorScript

const X509_cert_filename := "X509_Certificate.crt"
const X509_key_filename := "X509_Key.key"

# Server name
var CN := "FGAuth"
# Organization
var O := "IgnoC"
# Country code
var C := "PL"
var not_before := "20250101000000"
var not_after := "20260101000000"

func _run() -> void:
	var CNOC: String = "CN=%s,O=%s,C=%s" % [CN, O, C]
	print("Key for CNOC: ", CNOC)
	var crypto := Crypto.new()
	var crypto_key := crypto.generate_rsa(4096)
	var X509_cert = crypto.generate_self_signed_certificate(crypto_key, CNOC, not_before, not_after)
	X509_cert.save("res://certificates/" + X509_cert_filename)
	crypto_key.save("res://certificates/" + X509_key_filename)
	print("Done.")
