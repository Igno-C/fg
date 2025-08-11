use std::collections::HashMap;

pub mod benchmark;

pub trait Positioned {
    fn get_pos(&self) -> (i32, i32);
}

pub trait Positionable {
    fn set_pos(&mut self, x: i32, y: i32);
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

    pub fn insert(&mut self, o: T) {
        let pos = o.get_pos();
        let smallpos = (pos.0 / self.distance, pos.1 / self.distance);
        self.map.entry(smallpos).or_insert(Vec::new()).push(o);
    }

    pub fn get_adjacent(&self, pos: (i32, i32)) {
        let smallpos = (pos.0 / self.distance, pos.1 / self.distance);
        
    }
}
