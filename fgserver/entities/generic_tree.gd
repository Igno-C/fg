class_name GenericTree

extends GenericScriptedEntity


@export_enum("oak", "fir") var kind: String = "oak"

func _ready() -> void:
	related_scene = "tree"
	public_data["kind"] = kind
