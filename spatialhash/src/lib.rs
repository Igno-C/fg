use std::collections::HashMap;

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
}


pub struct SpatialHashChecker<T: Positioned + Positionable> {
    // topleft: (i32, i32),
    distance: i32,
    map: HashMap<(i32, i32), Vec<T>>
}

impl<T: Positioned + Positionable> SpatialHashChecker<T> {
    pub fn new(distance: i32) -> Self {
        Self {
            distance,
            map: HashMap::new(),
        }
    }

    fn get_smallpos(&self, pos: (i32, i32)) -> (i32, i32) {
        (pos.0.div_euclid(self.distance), pos.1.div_euclid(self.distance))
    }

    pub fn insert(&mut self, o: T) {
        let pos = o.get_pos();
        let smallpos = self.get_smallpos(pos);
        // println!("Inserting {pos:?} at {smallpos:?}");
        self.map.entry(smallpos).or_insert(Vec::new()).push(o);
    }

    pub fn for_each_adjacent<F: FnMut(&T) -> ()>(&self, pos: (i32, i32), mut closure: F) {
        let smallpos = self.get_smallpos(pos);
        const ADJACENT_RADIUS: i32 = 1;

        for xdelta in -ADJACENT_RADIUS..=ADJACENT_RADIUS {
            for ydelta in -ADJACENT_RADIUS..=ADJACENT_RADIUS {
                let checkpos = (smallpos.0 + xdelta, smallpos.1 + ydelta);
                if let Some(entities) = self.map.get(&checkpos) {
                    for entity in entities.iter() {
                        closure(entity);
                    }
                }
            }
        }
    }
}

impl<T: Positioned + Positionable + Identifiable> SpatialHashChecker<T> {
    pub fn remove(&mut self, e: T) {
        if let Some(entities) = self.map.get_mut(&e.get_pos()) {
            entities.retain(|ee| {
                ee.get_id() != e.get_id()
            });
        }
    }

    pub fn update_pos(&mut self, e: T, newpos: (i32, i32)) {
        
    }
}

pub struct SpatialFieldChecker<T: Positioned + Positionable> {
    distance: i32,
    topleft: (i32, i32),
    // bottomright: (i32, i32),

    objects: Vec<Vec<Vec<T>>>
}

impl<T: Positioned + Positionable> SpatialFieldChecker<T> {
    pub fn new(distance: i32, topleft: (i32, i32), bottomright: (i32, i32)) -> Self {
        let smalltopleft = (topleft.0.div_euclid(distance), topleft.1.div_euclid(distance));
        let smallbottomright = (bottomright.0.div_euclid(distance), bottomright.1.div_euclid(distance));

        let xsize = (smallbottomright.0 - smalltopleft.0) as usize + 1;
        let ysize = (smallbottomright.1 - smalltopleft.1) as usize + 1;

        let mut objects = Vec::with_capacity(xsize);
        for _ in 0..xsize {
            let mut yvec = Vec::with_capacity(ysize);
            for _ in 0..ysize {
                yvec.push(Vec::new());
            }
            objects.push(yvec);
        }

        Self {
            distance,
            topleft: smalltopleft,
            // bottomright: smallbottomright,
            objects
        }
    }

    fn get_smallpos(&self, pos: (i32, i32)) -> (i32, i32) {
        (pos.0.div_euclid(self.distance), pos.1.div_euclid(self.distance))
    }

    fn smallpos_to_indexes(&self, smallpos: (i32, i32)) -> (usize, usize) {
        let xpos = (smallpos.0 - self.topleft.0) as usize;
        let ypos = (smallpos.1 - self.topleft.1) as usize;

        (xpos, ypos)
    }

    fn pos_to_indexes(&self, pos: (i32, i32)) -> (usize, usize) {
        let smallpos = self.get_smallpos(pos);
        self.smallpos_to_indexes(smallpos)
    }

    pub fn insert(&mut self, o: T) {
        let pos = o.get_pos();
        let (xpos, ypos) = self.pos_to_indexes(pos);
        // println!("Inserting {pos:?} at ({xpos}, {ypos})");
        if let Some(yvec) = self.objects.get_mut(xpos) {
            if let Some(ovec) = yvec.get_mut(ypos) {
                ovec.push(o);
            }
        }
    }

    pub fn for_each_adjacent<F: FnMut(&T) -> ()>(&self, pos: (i32, i32), mut closure: F) {
        let smallpos = self.get_smallpos(pos);
        const ADJACENT_RADIUS: i32 = 1;

        for xdelta in -ADJACENT_RADIUS..=ADJACENT_RADIUS {
            for ydelta in -ADJACENT_RADIUS..=ADJACENT_RADIUS {
                let checkpos = (smallpos.0 + xdelta, smallpos.1 + ydelta);
                // println!("Really checking at {checkpos:?}");
                let (xpos, ypos) = self.smallpos_to_indexes(checkpos);
                // println!("Indexes of {xpos}, {ypos}");
                if let Some(yvec) = self.objects.get(xpos) {
                    if let Some(ovec) = yvec.get(ypos) {
                        for entity in ovec.iter() {
                            closure(entity);
                        }
                    }
                }
            }
        }
    }
}
