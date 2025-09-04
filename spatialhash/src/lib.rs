use std::{cell::RefCell, rc::Rc};
pub mod benchmark;
pub mod spatialhash;

pub struct TestObject {
    pub pos: (i32, i32),
    pub dummy_data: [u8; 400]
}

impl TestObject {
    pub fn new_rc(pos: (i32, i32)) -> Rc<RefCell<TestObject>>  {
        Rc::new(RefCell::new(
            TestObject {
                pos,
                dummy_data: [0; 400]
            }
        ))
    }
}

#[derive(Clone)]
pub struct LazyChecker {
    distance: i32,
    objects: Vec<(i32, Rc<RefCell<TestObject>>)>
}

impl LazyChecker {
    pub fn new(distance: i32) -> Self {
        Self {
            distance,
            objects: Vec::new()
        }
    }

    pub fn insert(&mut self, id: i32, o: Rc<RefCell<TestObject>>) {
        self.objects.push((id, o));
    }

    pub fn get_adjacent(&self, pos: (i32, i32)) -> impl Iterator<Item = &(i32, Rc<RefCell<TestObject>>)> {
        self.objects.iter().filter(move |o| {
            distance(o.1.borrow().pos, pos) <= self.distance
        })
    }

    pub fn get_object(&mut self, id: i32) -> &Rc<RefCell<TestObject>> {
        self.objects.iter().find_map(|o| {
            (o.0 == id).then_some(&o.1)
        }).unwrap()
    }

    pub fn remove(&mut self, id: i32) {
        let index = self.objects.iter().position(|o| o.0 == id).unwrap();
        self.objects.remove(index);
    }
}

fn distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

// pub struct SpatialHashChecker<T: Positioned + Positionable> {
//     // topleft: (i32, i32),
//     distance: i32,
//     map: HashMap<(i32, i32), Vec<T>>
// }

// impl<T: Positioned + Positionable> SpatialHashChecker<T> {
//     pub fn new(distance: i32) -> Self {
//         Self {
//             distance,
//             map: HashMap::new(),
//         }
//     }

//     fn get_smallpos(&self, pos: (i32, i32)) -> (i32, i32) {
//         (pos.0.div_euclid(self.distance), pos.1.div_euclid(self.distance))
//     }

//     pub fn insert(&mut self, o: T) {
//         let pos = o.get_pos();
//         let smallpos = self.get_smallpos(pos);
//         // println!("Inserting {pos:?} at {smallpos:?}");
//         self.map.entry(smallpos).or_insert(Vec::new()).push(o);
//     }

//     pub fn for_each_adjacent<F: FnMut(&T) -> ()>(&self, pos: (i32, i32), mut closure: F) {
//         let smallpos = self.get_smallpos(pos);
//         const ADJACENT_RADIUS: i32 = 1;

//         for xdelta in -ADJACENT_RADIUS..=ADJACENT_RADIUS {
//             for ydelta in -ADJACENT_RADIUS..=ADJACENT_RADIUS {
//                 let checkpos = (smallpos.0 + xdelta, smallpos.1 + ydelta);
//                 if let Some(entities) = self.map.get(&checkpos) {
//                     for entity in entities.iter() {
//                         closure(entity);
//                     }
//                 }
//             }
//         }
//     }
// }

// impl<T: Positioned + Positionable + Identifiable> SpatialHashChecker<T> {
//     pub fn remove(&mut self, e: T) {
//         if let Some(entities) = self.map.get_mut(&e.get_pos()) {
//             entities.retain(|ee| {
//                 ee.get_id() != e.get_id()
//             });
//         }
//     }

//     pub fn update_pos(&mut self, e: T, newpos: (i32, i32)) {
        
//     }
// }

// use godot::{builtin::Rect2i, global::godot_print};


// pub const GRID_SIZE: i32 = 8;
// pub const CHECK_RADIUS: i32 = 3;

fn get_smallpos(distance: i32, pos: (i32, i32)) -> (i32, i32) {
    (pos.0.div_euclid(distance), pos.1.div_euclid(distance))
}

#[derive(Default)]
pub struct SpatialHash {
    distance: i32,
    topleft: (i32, i32),
    width: usize,
    check_radius: i32,
    // bottomright: (i32, i32),

    map: Vec<Vec<i32>>
}

impl SpatialHash {
    // pub fn from_used_rect(rect: &Rect2i) -> Self {
    //     let rect_topleft = (rect.position.x, rect.position.y);
    //     let topleft = get_smallpos(rect_topleft);
    //     let rect_bottomright = (rect.end().x, rect.end().y);
    //     let bottomright = get_smallpos(rect_bottomright);

    //     let width = (bottomright.0 - topleft.0) as usize + 1;
    //     let height = (bottomright.1 - topleft.1) as usize + 1;

    //     let mut map = Vec::with_capacity(width*height);
    //     for _ in 0..width*height {
    //         map.push(Vec::new());
    //     }

    //     Self {
    //         topleft,
    //         width,
    //         map
    //     }
    // }

    

    pub fn new(distance: i32, topleft: (i32, i32), bottomright: (i32, i32), check_radius: i32) -> Self {
        let rect_topleft = topleft;
        let topleft = get_smallpos(distance, rect_topleft);
        let rect_bottomright = bottomright;
        let bottomright = get_smallpos(distance, rect_bottomright);

        let width = (bottomright.0 - topleft.0) as usize + 1;
        let height = (bottomright.1 - topleft.1) as usize + 1;

        let mut map = Vec::with_capacity(width*height);
        for _ in 0..width*height {
            map.push(Vec::new());
        }

        Self {
            distance,
            topleft,
            width,
            check_radius,
            map
        }
    }

    

    fn smallpos_to_index(&self, smallpos: (i32, i32)) -> usize {
        let xpos = (smallpos.0 - self.topleft.0) as usize;
        let ypos = (smallpos.1 - self.topleft.1) as usize;

        xpos + ypos * self.width
    }

    fn pos_to_index(&self, pos: (i32, i32)) -> usize {
        let smallpos = get_smallpos(self.distance, pos);
        self.smallpos_to_index(smallpos)
    }

    pub fn insert(&mut self, id: i32, pos: (i32, i32)) {
        let i = self.pos_to_index(pos);
        if let Some(idvec) = self.map.get_mut(i) {
            idvec.push(id);
        }
    }

    pub fn for_each_adjacent<F: FnMut(i32) -> ()>(&self, pos: (i32, i32), mut closure: F) {
        let smallpos = get_smallpos(self.distance, pos);

        for xdelta in -self.check_radius..=self.check_radius {
            for ydelta in -self.check_radius..=self.check_radius {
                let checkpos = (smallpos.0 + xdelta, smallpos.1 + ydelta);
                let i = self.smallpos_to_index(checkpos);
                if let Some(idvec) = self.map.get(i) {
                    for id in idvec.iter() {
                        closure(*id);
                    }
                }
            }
        }
    }

    pub fn remove(&mut self, id: i32, pos: (i32, i32)) -> Option<i32> {
        let smallpos = get_smallpos(self.distance, pos);
        let i = self.smallpos_to_index(smallpos);
        if let Some(objects) = self.map.get_mut(i) {
            if let Some(found_i) = objects.iter().position(|_id| {
                *_id != id
            }) {
                return Some(objects.remove(found_i));
            }
        }
        return None;
    }

    pub fn update_pos(&mut self, id: i32, oldpos: (i32, i32), newpos: (i32, i32)) {
        let oldsmallpos = get_smallpos(self.distance, oldpos);
        let newsmallpos = get_smallpos(self.distance, newpos);

        if oldsmallpos == newsmallpos {return}

        let oldi = self.smallpos_to_index(oldsmallpos);
        let newi = self.smallpos_to_index(newsmallpos);

        // godot_print!("Id {id} moved from {oldpos:?} to {newpos:?}, crossing grid boundary");

        if let Some(ids) = self.map.get_mut(oldi) {
            if let Some(index) = ids.iter().position(|_id| *_id == id) {
                ids.remove(index);
            }
        }

        if let Some(ids) = self.map.get_mut(newi) {
            ids.push(id);
        }
    }
}