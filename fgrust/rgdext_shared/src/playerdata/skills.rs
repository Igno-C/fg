use bitcode::{Decode, Encode};
// use serde::{Serialize, Deserialize};


// #[derive(Decode, Encode)]
#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Skill {
    Woodcutting,
    Mining,
    Smelting,
    Crafting,
    Farming,
    Strength,
    Agility,
    Endurance,
    Magic,
    Ranged
}
const NUM_SKILLS: usize = 10;

impl Skill {
    pub fn try_from_str(s: &str) -> Option<Skill> {
        match s {
            "woodcutting" => Some(Skill::Woodcutting),
            "mining" => Some(Skill::Mining),
            "smelting" => Some(Skill::Smelting),
            "crafting" => Some(Skill::Crafting),
            "farming" => Some(Skill::Farming),
            "strength" => Some(Skill::Strength),
            "agility" => Some(Skill::Agility),
            "endurance" => Some(Skill::Endurance),
            "magic" => Some(Skill::Magic),
            "ranged" => Some(Skill::Ranged),
            _ => None,
        }
    }

    pub fn skill_strs() -> [&'static str; NUM_SKILLS] {
        [
            "woodcutting",
            "mining",
            "smelting",
            "crafting",
            "farming",
            "strength",
            "agility",
            "endurance",
            "magic",
            "ranged",
        ]
    }
}

#[derive(Clone, Encode, Decode, Debug)]
pub struct Skills {
    skills: [u8; NUM_SKILLS],
}

impl std::ops::Index<Skill> for Skills {
    type Output = u8;

    fn index(&self, index: Skill) -> &u8 {
        &self.skills[index as usize]
    }
}

impl std::ops::IndexMut<Skill> for Skills {
    fn index_mut(&mut self, index: Skill) -> &mut u8 {
        &mut self.skills[index as usize]
    }
}

impl Default for Skills {
    fn default() -> Self {
        Self{skills: [1; NUM_SKILLS]}
    }
}

#[derive(Clone, Default, Encode, Decode, Debug)]
pub struct SkillProgress {
    skill_progress: [i32; NUM_SKILLS],
}

impl std::ops::Index<Skill> for SkillProgress {
    type Output = i32;

    fn index(&self, index: Skill) -> &i32 {
        &self.skill_progress[index as usize]
    }
}

impl std::ops::IndexMut<Skill> for SkillProgress {
    fn index_mut(&mut self, index: Skill) -> &mut i32 {
        &mut self.skill_progress[index as usize]
    }
}