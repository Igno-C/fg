extends Control

@onready var skill_icon: TextureRect = $HBoxContainer/Centerer/SkillIcon
@onready var top_label: Label = $HBoxContainer/VBoxContainer/TopLabel
@onready var bottom_label: Label = $HBoxContainer/VBoxContainer/BottomLabel

var skill: String

func set_skill(skill: String) -> void:
	var texture: Texture2D = load("res://graphics/icons/%s.png" % skill)
	skill_icon.texture = texture
	
	top_label.text = skill.capitalize()

func set_stats(level: int, xp: int) -> void:
	bottom_label.text = "%s/100" % level
	if xp != 0: 
		bottom_label.tooltip_text = "Xp: %s" % xp
