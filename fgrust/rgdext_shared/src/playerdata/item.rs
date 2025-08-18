use bitcode::{Encode, Decode};

#[derive(Clone, Encode, Decode, Debug)]
pub enum Item {
    TestItem{test_value: i32}
}