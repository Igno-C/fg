use bitcode::{Encode, Decode};

#[derive(Clone, Encode, Decode)]
pub enum Item {
    TestItem{test_value: i32}
}