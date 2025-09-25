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
        let mut custom_ints = Vec::new();
        let mut custom_floats = Vec::new();
        let mut custom_strings = Vec::new();

        for (k, v) in self.custom_data.iter_shared() {
            if let Ok(skey) = k.try_to_relaxed::<String>() {
                if let Ok(ivar) = v.try_to_relaxed::<i32>() {
                    custom_ints.push((skey, ivar));
                }
                else if let Ok(fvar) = v.try_to_relaxed::<f32>() {
                    custom_floats.push((skey, fvar));
                }
                else if let Ok(svar) = v.try_to_relaxed::<String>() {
                    custom_strings.push((skey, svar));
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

    #[func]
    /// If successfully combined, returns null.
    /// 
    /// Otherwise, returns the other item that was attempted to combine into this one.
    fn try_combine(&mut self, other: Gd<ItemResource>) -> Option<Gd<ItemResource>> {
        let ob = other.bind();
        if self.id_string != ob.id_string || !self.stackable {
            drop(ob);
            return Some(other);
        }
        else {
            self.count += ob.count;
            return None;
        }
    }

    #[func]
    /// If successfully split, returns the newly split item, this item's count is substracted.
    /// 
    /// Otherwise, returns null. Can't reduce this item's count to zero.
    fn try_split(&mut self, new_count: i32) -> Option<Gd<ItemResource>> {
        if self.count <= new_count || !self.stackable {
            return None;
        }
        else {
            let split_item: Gd<ItemResource> = Gd::from_init_fn(|base| {
                ItemResource {
                    id_string: self.id_string.clone(),
                    name: self.name.clone(),
                    description: self.description.clone(),
                    stackable: self.stackable.clone(),
                    count: new_count,
                    custom_data: self.custom_data.clone(),
                    base
                }
            });
            self.count -= new_count;

            return Some(split_item);
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
    custom_ints: Vec<(String, i32)>,
    custom_floats: Vec<(String, f32)>,
    custom_strings: Vec<(String, String)>,
}

impl Item {
    pub fn to_resource(&self) -> Gd<ItemResource> {
        let mut custom_data = Dictionary::new();

        for (key, value) in &self.custom_ints {
            custom_data.set(key.as_str(), *value);
        }
        for (key, value) in &self.custom_floats {
            custom_data.set(key.as_str(), *value);
        }
        for (key, value) in &self.custom_strings {
            custom_data.set(key.as_str(), value.as_str());
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

    pub fn try_combine(&mut self, other: Item) -> Option<Item> {
        if self.id_string != other.id_string || !self.stackable {
            return Some(other);
        }
        else {
            self.count += other.count;
            return None;
        }
    }

    pub fn try_split(&mut self, new_count: i32) -> Option<Item> {
        if self.count <= new_count || !self.stackable {
            return None;
        }
        else {
            let mut split_item = self.clone();
            split_item.count = new_count;
            self.count -= new_count;

            return Some(split_item);
        }
    }
}
