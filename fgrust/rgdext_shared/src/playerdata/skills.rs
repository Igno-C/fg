use bitcode::{Decode, Encode};
// use serde::{Serialize, Deserialize};


// #[derive(Decode, Encode)]
#[repr(usize)]
pub enum Skill {
    WoodCutting,
    Mining,
    Smelting,
    Crafting,
    Farming,
}
const NUM_SKILLS: usize = 5;

#[derive(Clone, Default, Encode, Decode, Debug)]
pub struct Skills {
    skills: [u8; NUM_SKILLS]
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
