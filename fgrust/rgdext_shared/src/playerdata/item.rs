use std::collections::HashMap;

use godot::prelude::*;
use bitcode::{Encode, Decode};


#[derive(GodotClass)]
#[class(base=Resource)]
pub struct ItemResource {
    #[export]
    /// Unique string id that uniquely identifies this item, also used for the icon on client-side.
    /// 
    /// Items of the same id_string get stacked together.
    id_string: GString,
    #[export]
    name: GString,
    #[export]
    description: GString,
    #[export]
    stackable: bool,
    #[export]
    count: i32,
    #[export]
    custom_data: Dictionary,

    base: Base<Resource>,
}

#[godot_api]
impl IResource for ItemResource {
    fn init(base: Base<Resource>) -> Self {
        Self {
            id_string: "".into(),
            name: "".into(),
            description: "".into(),
            stackable: true,
            count: 1,
            custom_data: Dictionary::new(),
            base,
        }
    }
}

#[godot_api]
impl ItemResource {
    pub fn to_item(&self) -> Item {
        let mut custom_ints = HashMap::new();
        let mut custom_floats = HashMap::new();
        let mut custom_strings = HashMap::new();

        for (k, v) in self.custom_data.iter_shared() {
            if let Ok(skey) = k.try_to_relaxed::<String>() {
                if let Ok(ivar) = v.try_to_relaxed::<i32>() {
                    custom_ints.insert(skey, ivar);
                }
                else if let Ok(fvar) = v.try_to_relaxed::<f32>() {
                    custom_floats.insert(skey, fvar);
                }
                else if let Ok(svar) = v.try_to_relaxed::<String>() {
                    custom_strings.insert(skey, svar);
                }
                else {
                    godot_error!("Non-string, non-int value in ItemResource!");
                }
            }
            else {
                godot_error!("Non-string key in ItemResource!");
            }
        }

        Item {
            id_string: self.id_string.to_string(),
            name: self.name.to_string(),
            description: self.description.to_string(),
            stackable: self.stackable,
            count: self.count,
            custom_ints,
            custom_floats,
            custom_strings
        }
    }
}

#[derive(Clone, Encode, Decode, Debug)]
pub struct Item {
    id_string: String,
    name: String,
    description: String,
    stackable: bool,
    pub count: i32,
    custom_ints: HashMap<String, i32>,
    custom_floats: HashMap<String, f32>,
    custom_strings: HashMap<String, String>,
}

impl Item {
    pub fn to_resource(&self) -> Gd<ItemResource> {
        let mut custom_data = Dictionary::new();

        for (key, value) in &self.custom_ints {
            custom_data.set(key.clone(), *value);
        }
        for (key, value) in &self.custom_floats {
            custom_data.set(key.clone(), *value);
        }
        for (key, value) in &self.custom_strings {
            custom_data.set(key.clone(), value.clone());
        }

        Gd::from_init_fn(|a| {
            ItemResource {
                id_string: GString::from(&self.id_string),
                name: GString::from(&self.name),
                description: GString::from(&self.description),
                stackable: self.stackable,
                count: self.count,
                custom_data,

                base: a
            }
        })
    }

    pub fn id_string(&self) -> &str {
        &self.id_string
    }

    pub fn stackable(&self) -> bool {
        self.stackable
    }
}
