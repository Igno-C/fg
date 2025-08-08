use godot::prelude::*;
use std::{fs::File, io::{Read, Write}, path::PathBuf};


struct FileDb {
    basepath: PathBuf
}

impl FileDb {
    pub fn new() -> Self {
        Self {
            basepath: PathBuf::from("./data/")
        }
    }

    pub fn new_with_dir(path: &str) -> Self {
        Self {
            basepath: PathBuf::from(path)
        }
    }

    fn create_or_truncate_file(&self, p: &str) -> File {
        let path = self.basepath.join(p);

        File::create(path).unwrap()
    }

    // pub fn create_id(&self, id: i32) {
    //     self.write_id(id, &[]);
    // }

    pub fn write_id(&self, id: i32, data: &[u8]) {
        let mut file = self.create_or_truncate_file(&id.to_string());

        file.write_all(data).unwrap();
    }

    pub fn read(&self, id: i32) -> Vec<u8> {
        let mut file = self.create_or_truncate_file(&id.to_string());

        let mut data = Vec::with_capacity(1000);
        file.read_to_end(&mut data).unwrap();

        data
    }
}


#[derive(GodotClass)]
#[class(base=RefCounted)]
pub struct FileDbAccessor {
    filedb: FileDb,

    base: Base<RefCounted>
}

#[godot_api]
impl IRefCounted for FileDbAccessor {
    fn init(base: Base<RefCounted>) -> Self {
        FileDbAccessor {
            filedb: FileDb::new(),

            base
        }
    }
}

#[godot_api]
impl FileDbAccessor {
    #[func]
    fn new_with_dir(path: String) -> Gd<Self> {
        Gd::from_init_fn(|base| {
            Self {
                filedb: FileDb::new_with_dir(&path),

                base
            }
        })
    }

    #[func]
    fn write(&self, id: i32, data: PackedByteArray) {
        self.filedb.write_id(id, data.as_slice());
    }

    #[func]
    fn read(&self, id: i32) -> PackedByteArray {
        let data = self.filedb.read(id);
        PackedByteArray::from(data)
    }
}
