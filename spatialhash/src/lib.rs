use std::ops::Index;

pub mod benchmark;

pub trait Positioned {
    fn get_pos(&self) -> (i32, i32);
}

pub trait Positionable {
    fn set_pos(&mut self, x: i32, y: i32);
}

pub trait Identifiable {
    fn get_id(&self) -> impl Eq;
}

impl<T: Positioned> Positioned for &T {
    fn get_pos(&self) -> (i32, i32) {
        (*self).get_pos()
    }
}

impl Positioned for (i32, i32) {
    fn get_pos(&self) -> (i32, i32) {
        *self
    }
}

impl Positionable for (i32, i32) {
    fn set_pos(&mut self, x: i32, y: i32) {
        *self = (x, y);
    }
}

fn distance(o1: impl Positioned, o2: impl Positioned) -> i32 {
    let (x1, y1) = o1.get_pos();
    let (x2, y2) = o2.get_pos();

    (x1-x2).abs().max((y1-y2).abs())
}

pub struct LazyChecker<T: Positioned + Positionable> {
    distance: i32,
    objects: Vec<T>
}

impl<T: Positioned + Positionable> LazyChecker<T> {
    pub fn new(distance: i32) -> Self {
        Self {
            distance,
            objects: Vec::new()
        }
    }

    pub fn insert(&mut self, o: T) {
        self.objects.push(o);
    }

    pub fn get_adjacent(&self, pos: (i32, i32)) -> impl Iterator<Item = &T> {
        self.objects.iter().filter(move |o| {
            distance(o, &pos) <= self.distance
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.objects.iter_mut()
    }
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

pub struct SpatialFieldChecker<T: Positioned + Positionable> {
    distance: i32,
    topleft: (i32, i32),
    width: usize,
    check_radius: i32,
    // bottomright: (i32, i32),

    objects: Vec<Vec<T>>
}

impl<T: Positioned + Positionable> SpatialFieldChecker<T> {
    pub fn new(distance: i32, topleft: (i32, i32), bottomright: (i32, i32), check_radius: i32) -> Self {
        let smalltopleft = (topleft.0.div_euclid(distance), topleft.1.div_euclid(distance));
        let smallbottomright = (bottomright.0.div_euclid(distance), bottomright.1.div_euclid(distance));

        let xsize = (smallbottomright.0 - smalltopleft.0) as usize + 1;
        let ysize = (smallbottomright.1 - smalltopleft.1) as usize + 1;


        let mut objects = Vec::with_capacity(xsize);
        for _ in 0..(xsize*ysize) {
            objects.push(Vec::new());
        }

        Self {
            distance,
            topleft: smalltopleft,
            width: xsize,
            check_radius,
            objects
        }
    }

    fn get_smallpos(&self, pos: (i32, i32)) -> (i32, i32) {
        (pos.0.div_euclid(self.distance), pos.1.div_euclid(self.distance))
    }

    fn smallpos_to_index(&self, smallpos: (i32, i32)) -> usize {
        let xpos = (smallpos.0 - self.topleft.0) as usize;
        let ypos = (smallpos.1 - self.topleft.1) as usize;

        xpos + ypos * self.width
    }

    fn pos_to_index(&self, pos: (i32, i32)) -> usize {
        let smallpos = self.get_smallpos(pos);
        self.smallpos_to_index(smallpos)
    }

    pub fn insert(&mut self, o: T) {
        let pos = o.get_pos();
        let i = self.pos_to_index(pos);
        if let Some(ovec) = self.objects.get_mut(i) {
            ovec.push(o);
        }
    }

    pub fn for_each_adjacent<F: FnMut(&T) -> ()>(&self, pos: (i32, i32), mut closure: F) {
        let smallpos = self.get_smallpos(pos);

        for xdelta in -self.check_radius..=self.check_radius {
            for ydelta in -self.check_radius..=self.check_radius {
                let checkpos = (smallpos.0 + xdelta, smallpos.1 + ydelta);
                let i = self.smallpos_to_index(checkpos);
                if let Some(ovec) = self.objects.get(i) {
                    for entity in ovec.iter() {
                        closure(entity);
                    }
                }
            }
        }
    }
}

impl<T: Positioned + Positionable + Identifiable> SpatialFieldChecker<T> {
    pub fn remove(&mut self, e: T) -> Option<T> {
        let smallpos = self.get_smallpos(e.get_pos());
        let i = self.smallpos_to_index(smallpos);
        if let Some(objects) = self.objects.get_mut(i) {
            if let Some(found_i) = objects.iter().position(|o| {
                o.get_id() != e.get_id()
            }) {
                return Some(objects.remove(found_i));
            }
        }
        return None;
    }

    // pub fn update_pos(&mut self, e: T, newpos: (i32, i32)) -> bool {
    //     let smallpos = self.get_smallpos(e.get_pos());
    //     let i = self.smallpos_to_index(smallpos);

    //     if let Some(objects) = self.objects.get_mut(i) {
    //         if let Some(object) = objects.iter_mut().find(|o| o.get_id() == e.get_id()) {
    //             object.set_pos(newpos.0, newpos.1);
    //             let newsmallpos = self.get_smallpos(newpos);

    //             if newsmallpos != smallpos {
    //                 let old_i = self.smallpos_to_index(smallpos);
    //                 if let Some(bucket) = self.objects.get_mut(old_i) {
    //                     if let Some(found_i) = bucket.iter().position(|o| {o.get_id() != e.get_id()}) {
    //                         let o = objects.remove(found_i);
    //                         let new_i = self.smallpos_to_index(newsmallpos);
    //                         self.objects.get_mut(new_i).map(|| {

    //                         });
    //                     }
    //                 }
    //                 // moved_buckets = true;
    //                 if let Some(o) = self.remove(e) {
    //                     self.insert(o);
    //                 }
    //             }
    //         }
    //     }
    // }
}