pub const GRID_SIZE: i32 = 8;
pub const CHECK_RADIUS: i32 = 3;

#[derive(Clone)]
pub struct SpatialHash<I: Eq, T> {
    grid_size: i32,
    topleft: (i32, i32),
    width: usize,
    height: usize,
    check_radius: i32,

    map: Vec<Vec<(I, T)>>
}

impl<I: Eq, T> Default for SpatialHash<I, T> {
    fn default() -> Self {
        Self {
            grid_size: 0,
            topleft: (0, 0),
            width: 0,
            height: 0,
            check_radius: 0,
            map: Vec::new()
        }
    }
}

pub enum MoveDelta {
    Delta{from: (i32, i32), to: (i32, i32), check_radius: i32},
    NoMove
}

impl<I: Eq, T> SpatialHash<I, T> {
    pub fn new(grid_size: i32, topleft: (i32, i32), bottomright: (i32, i32), check_radius: i32) -> Self {
        let smalltopleft = (topleft.0.div_euclid(grid_size), topleft.1.div_euclid(grid_size));
        let smallbottomright = (bottomright.0.div_euclid(grid_size), bottomright.1.div_euclid(grid_size));

        let xsize = (smallbottomright.0 - smalltopleft.0) as usize + 1;
        let ysize = (smallbottomright.1 - smalltopleft.1) as usize + 1;


        let mut map = Vec::with_capacity(xsize);
        for _ in 0..(xsize*ysize) {
            map.push(Vec::new());
        }

        Self {
            grid_size,
            topleft: smalltopleft,
            width: xsize,
            height: ysize,
            check_radius,
            map
        }
    }

    fn get_smallpos(&self, pos: (i32, i32)) -> (i32, i32) {
        (pos.0.div_euclid(self.grid_size), pos.1.div_euclid(self.grid_size))
    }

    fn smallpos_to_index(&self, smallpos: (i32, i32)) -> Option<usize> {
        if smallpos.0 < self.topleft.0 || smallpos.1 < self.topleft.1 {
            return None
        }

        let xpos = (smallpos.0 - self.topleft.0) as usize;
        let ypos = (smallpos.1 - self.topleft.1) as usize;

        if xpos >= self.width || ypos >= self.height {
            return None;
        }

        return Some(xpos + ypos * self.width);
    }

    fn pos_to_index(&self, pos: (i32, i32)) -> Option<usize> {
        let smallpos = self.get_smallpos(pos);
        self.smallpos_to_index(smallpos)
    }

    pub fn insert(&mut self, id: I, object: T, pos: (i32, i32)) {
        if let Some(i) = self.pos_to_index(pos) {
            self.map[i].push((id, object));
        }
    }

    pub fn for_each_adjacent<F: FnMut(&(I, T)) -> ()>(&self, pos: (i32, i32), mut closure: F) {
        let smallpos = self.get_smallpos(pos);

        for xdelta in -self.check_radius..=self.check_radius {
            for ydelta in -self.check_radius..=self.check_radius {
                let checkpos = (smallpos.0 + xdelta, smallpos.1 + ydelta);
                if let Some(i) = self.smallpos_to_index(checkpos) {
                    for id in self.map[i].iter() {
                        closure(id);
                    }
                }
            }
        }
    }

    pub fn remove(&mut self, net_id: I, pos: (i32, i32)) -> Option<(I, T)> {
        let smallpos = self.get_smallpos(pos);
        // let i = ;
        if let Some(i) = self.smallpos_to_index(smallpos) {
            let objects = &mut self.map[i];
            if let Some(found_i) = objects.iter().position(|_id| {
                _id.0 == net_id
            }) {
                return Some(objects.remove(found_i));
            }
        }
        return None;
    }

    pub fn update_pos<'a>(&'a mut self, id: I, oldpos: (i32, i32), newpos: (i32, i32)) -> MoveDelta {
        let oldsmallpos = self.get_smallpos(oldpos);
        let newsmallpos = self.get_smallpos(newpos);

        if oldsmallpos == newsmallpos {return MoveDelta::NoMove;}

        let oldi_maybe = self.smallpos_to_index(oldsmallpos);
        let newi_maybe = self.smallpos_to_index(newsmallpos);

        if let Some(oldi) = oldi_maybe && let Some(newi) = newi_maybe {
            if let Some(remove_index) = self.map[oldi].iter().position(|_id| _id.0 == id) {
                let id = self.map[oldi].remove(remove_index);

                self.map[newi].push(id);
                return MoveDelta::Delta{from: oldsmallpos, to: newsmallpos, check_radius: self.check_radius};
            }
        }
        return MoveDelta::NoMove;
    }

    pub fn get(&self, pos: (i32, i32), id: I) -> Option<&T> {
        if let Some(i) = self.smallpos_to_index(self.get_smallpos(pos)) {
            for object in &self.map[i] {
                if object.0 == id {
                    return Some(&object.1)
                }
            }
        }
        return None;
    }
}

impl MoveDelta {
    pub fn for_each_with<'a, I: Eq, T, F: FnMut(&(I, T)) -> ()>(&self, hash: &'a SpatialHash<I, T>, mut closure: F) {
        let (from, to, radius) = match self {
            MoveDelta::Delta{from, to, check_radius} => (*from, *to, *check_radius),
            MoveDelta::NoMove => return,
        };

        for xdelta in -radius..=radius {
            for ydelta in -radius..=radius {
                let checkpos = (to.0 + xdelta, to.1 + ydelta);
                let from_distance = from.0.abs_diff(checkpos.0).max(from.1.abs_diff(checkpos.1)) as i32;
                if from_distance > radius {
                    if let Some(index) = hash.smallpos_to_index(checkpos) {
                        for o in hash.map[index].iter() {
                            closure(o);
                        }
                    }
                }
            }
        }
    }
}
