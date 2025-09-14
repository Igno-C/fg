use godot::builtin::Rect2i;


pub const GRID_SIZE: i32 = 8;
pub const CHECK_RADIUS: i32 = 3;

pub struct SpatialHash<I: Eq + Copy, T> {
    grid_size: i32,
    topleft: (i32, i32),
    width: usize,
    height: usize,
    check_radius: i32,

    map: Vec<Vec<(I, T)>>
}

impl<I: Eq + Copy, T> Default for SpatialHash<I, T> {
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

pub enum MoveDelta<I: Eq + Copy> {
    Delta{from: (i32, i32), to: (i32, i32), check_radius: i32, exclude_id: I},
    NoMove
}

impl<I: Eq + Copy, T> SpatialHash<I, T> {
    pub fn from_used_rect_default(rect: &Rect2i) -> Self {
        let rect_topleft = (rect.position.x, rect.position.y);
        let rect_bottomright = (rect.end().x, rect.end().y);

        Self::new(GRID_SIZE, rect_topleft, rect_bottomright, CHECK_RADIUS)
    }

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

    pub fn update_pos<'a>(&'a mut self, id: I, oldpos: (i32, i32), newpos: (i32, i32)) -> MoveDelta<I> {
        let oldsmallpos = self.get_smallpos(oldpos);
        let newsmallpos = self.get_smallpos(newpos);

        if oldsmallpos == newsmallpos {return MoveDelta::NoMove;}

        let oldi_maybe = self.smallpos_to_index(oldsmallpos);
        let newi_maybe = self.smallpos_to_index(newsmallpos);

        if let Some(oldi) = oldi_maybe && let Some(newi) = newi_maybe {
            if let Some(remove_index) = self.map[oldi].iter().position(|_id| _id.0 == id) {
                let object = self.map[oldi].remove(remove_index);
                let id = object.0;
                self.map[newi].push(object);
                return MoveDelta::Delta{from: oldsmallpos, to: newsmallpos, check_radius: self.check_radius, exclude_id: id};
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

    pub fn get_mut(&mut self, pos: (i32, i32), id: I) -> Option<&mut T> {
        if let Some(i) = self.smallpos_to_index(self.get_smallpos(pos)) {
            for object in &mut self.map[i] {
                if object.0 == id {
                    return Some(&mut object.1)
                }
            }
        }
        return None;
    }
}

impl<I: Eq + Copy> MoveDelta<I> {
    pub fn for_each_with<'a, T, F: FnMut(&(I, T)) -> ()>(&self, hash: &'a SpatialHash<I, T>, mut closure: F) {
        let (from, to, radius, exclude_id) = match self {
            MoveDelta::Delta{from, to, check_radius, exclude_id} => (*from, *to, *check_radius, *exclude_id),
            MoveDelta::NoMove => return,
        };

        for xdelta in -radius..=radius {
            for ydelta in -radius..=radius {
                let checkpos = (to.0 + xdelta, to.1 + ydelta);
                let from_distance = from.0.abs_diff(checkpos.0).max(from.1.abs_diff(checkpos.1)) as i32;
                if from_distance > radius {
                    if let Some(index) = hash.smallpos_to_index(checkpos) {
                        for o in hash.map[index].iter() {
                            if o.0 != exclude_id {
                                closure(o);
                            }
                        }
                    }
                }
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::{RefCell}, rc::Rc};

    use crate::playerdata::PlayerData;

    fn create_player_data() -> Rc<RefCell<PlayerData>> {
        Rc::new(RefCell::new(PlayerData::from_name("".into(), -1)))
    }

    #[test]
    fn test_new_empty_grid() {
        let hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (0, 0), (31, 31), 3);
        
        assert_eq!(hash.grid_size, 8);
        assert_eq!(hash.topleft, (0, 0));
        // 0/8 = 0, 31/8 = 3
        // 0, 1, 2, 3 -> 4 width
        assert_eq!(hash.width, 4);
        assert_eq!(hash.check_radius, 3);
        assert_eq!(hash.map.len(), 16); // 4 * 4 = 16 cells
    }

    #[test]
    fn test_smallpos_conversion() {
        let hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (0, 0), (31, 31), 3);
        
        assert_eq!(hash.get_smallpos((0, 0)), (0, 0));
        assert_eq!(hash.get_smallpos((7, 7)), (0, 0));
        assert_eq!(hash.get_smallpos((8, 8)), (1, 1));
        assert_eq!(hash.get_smallpos((15, 15)), (1, 1));
        
        assert_eq!(hash.get_smallpos((-1, -1)), (-1, -1));
        assert_eq!(hash.get_smallpos((-8, -17)), (-1, -3));
        assert_eq!(hash.get_smallpos((-9, -16)), (-2, -2));
    }

    #[test]
    fn test_index_conversion() {
        let hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (0, 0), (31, 31), 3);
        
        assert_eq!(hash.smallpos_to_index((0, 0)), Some(0));
        assert_eq!(hash.smallpos_to_index((3, 0)), Some(3));
        assert_eq!(hash.smallpos_to_index((0, 3)), Some(12));
        assert_eq!(hash.smallpos_to_index((3, 3)), Some(15));

        assert_eq!(hash.smallpos_to_index((-1, 0)), None);
        assert_eq!(hash.smallpos_to_index((0, -11)), None);
        assert_eq!(hash.smallpos_to_index((3, -2)), None);
        assert_eq!(hash.smallpos_to_index((-1, 2)), None);


        assert_eq!(hash.smallpos_to_index((4, 3)), None);
    }

    #[test]
    fn test_insert_and_query_single_element() {
        let mut hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (0, 0), (31, 31), 3);
        let player_data = create_player_data();
        
        hash.insert(42, player_data, (10, 10));
        
        // Check that element was inserted correctly
        let index = hash.pos_to_index((10, 10)).unwrap();
        assert_eq!(hash.map[index].len(), 1);
        assert_eq!(hash.map[index][0].0, 42);
        
        // Verify it can be found when querying adjacent cells
        let mut found_ids = Vec::new();
        hash.for_each_adjacent((10, 10), |item| {
            found_ids.push(item.0);
        });
        
        assert_eq!(found_ids.len(), 1);
        assert_eq!(found_ids[0], 42);
    }

    #[test]
    fn test_insert_multiple_elements_same_cell() {
        let mut hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (0, 0), (31, 31), 3);
        let player_data = create_player_data();
        
        hash.insert(1, player_data.clone(), (10, 10));
        hash.insert(2, player_data, (11, 10));
        
        let index = hash.pos_to_index((10, 9)).unwrap();
        assert_eq!(hash.map[index].len(), 2);
        assert_eq!(hash.map[index][0].0, 1);
        assert_eq!(hash.map[index][1].0, 2);
    }

    #[test]
    fn test_insert_different_cells() {
        let mut hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (0, 0), (31, 31), 3);
        let player_data = create_player_data();
        
        hash.insert(1, player_data.clone(), (10, 10));   // Cell (1,1)
        hash.insert(2, player_data, (20, 20));   // Cell (2,2)
        
        let index1 = hash.pos_to_index((10, 10)).unwrap();
        let index2 = hash.pos_to_index((20, 20)).unwrap();
        
        assert_eq!(hash.map[index1].len(), 1);
        assert_eq!(hash.map[index2].len(), 1);
        assert_eq!(hash.map[index1][0].0, 1);
        assert_eq!(hash.map[index2][0].0, 2);
    }

    #[test]
    fn test_remove_existing_element() {
        let mut hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (0, 0), (31, 31), 3);
        let player_data = create_player_data();
        
        hash.insert(42, player_data, (10, 10));
        let removed = hash.remove(42, (10, 10));
        
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().0, 42);
        
        let index = hash.pos_to_index((10, 10)).unwrap();
        assert_eq!(hash.map[index].len(), 0);
    }

    #[test]
    fn test_remove_nonexistent_element() {
        let mut hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (0, 0), (31, 31), 3);
        let player_data = create_player_data();
        
        hash.insert(42, player_data, (10, 10));
        let removed = hash.remove(99, (10, 10));
        
        assert!(removed.is_none());
        // Should still have the original element
        let index = hash.pos_to_index((10, 10)).unwrap();
        assert_eq!(hash.map[index].len(), 1);
        assert_eq!(hash.map[index][0].0, 42);
    }

    #[test]
    fn test_update_position_same_cell() {
        let mut hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (0, 0), (31, 31), 3);
        let player_data = create_player_data();
        
        hash.insert(42, player_data, (10, 10));
        hash.update_pos(42, (10, 10), (15, 15)); // Same cell
        
        let index = hash.pos_to_index((10, 10)).unwrap();
        assert_eq!(hash.map[index].len(), 1);
        assert_eq!(hash.map[index][0].0, 42);
    }

    #[test]
    fn test_update_position_different_cells() {
        let mut hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (0, 0), (31, 31), 3);
        let player_data = create_player_data();
        
        hash.insert(42, player_data, (10, 10));   // Cell (1,1)
        hash.update_pos(42, (10, 10), (20, 20));  // Cell (2,2)
        
        let old_index = hash.pos_to_index((10, 10)).unwrap();
        let new_index = hash.pos_to_index((20, 20)).unwrap();
        
        assert_eq!(hash.map[old_index].len(), 0);
        assert_eq!(hash.map[new_index].len(), 1);
        assert_eq!(hash.map[new_index][0].0, 42);
    }

    #[test]
    fn test_query_adjacent_elements() {
        let mut hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (0, 0), (31, 31), 3);
        let player_data = create_player_data();
        
        // Insert elements in cells that should be adjacent to center cell
        hash.insert(1, player_data.clone(), (10, 10));   // Cell (1,1) 
        hash.insert(2, player_data.clone(), (20, 20));   // Cell (2,2)
        hash.insert(3, player_data, (5, 5));     // Cell (0,0)
        
        let mut found_ids = Vec::new();
        hash.for_each_adjacent((15, 15), |item| {
            found_ids.push(item.0);
        });
        
        // Should find all three elements since they're within check_radius of (1,1)
        assert_eq!(found_ids.len(), 3);
        assert!(found_ids.contains(&1));
        assert!(found_ids.contains(&2));
        assert!(found_ids.contains(&3));
    }

    #[test]
    fn test_query_adjacent_no_elements() {
        let mut hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (0, 0), (31, 31), 1);
        let player_data = create_player_data();
        
        // Insert elements in cells that should be adjacent to center cell
        hash.insert(1, player_data.clone(), (10, 10));   // Cell (1,1) 
        hash.insert(2, player_data.clone(), (20, 20));   // Cell (2,2)
        hash.insert(3, player_data, (5, 5));     // Cell (0,0)
        
        let mut found_ids = Vec::new();
        hash.for_each_adjacent((5, 6), |item| {
            found_ids.push(item.0);
        });
        
        assert_eq!(found_ids.len(), 2);
        assert!(found_ids.contains(&1));
        assert!(!found_ids.contains(&2)); // (2, 2 is outside of check radius of 1 around 0, 0)
        assert!(found_ids.contains(&3));
    }

    #[test]
    fn test_negative_coordinates() {
        let mut hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (-16, -16), (15, 15), 3);
        let player_data = create_player_data();
        
        // Test negative coordinates
        hash.insert(42, player_data, (-10, -10));  // Should be in cell (-2,-2)
        
        let index = hash.smallpos_to_index((-2, -2)).unwrap();
        assert_eq!(hash.map[index].len(), 1);
        assert_eq!(hash.map[index][0].0, 42);
    }

    #[test]
    fn test_pos_to_index_out_of_bounds() {
        let hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (0, 0), (31, 31), 3);
        
        assert_eq!(hash.pos_to_index((-1, 0)), None);
        assert_eq!(hash.pos_to_index((0, -1)), None);
        assert_eq!(hash.pos_to_index((32, 0)), None);
        assert_eq!(hash.pos_to_index((0, 32)), None);
    }

    #[test]
    fn test_get_existing_element() {
        let mut hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (0, 0), (31, 31), 3);
        let player_data = create_player_data();
        
        hash.insert(42, player_data.clone(), (10, 10));
        
        let result = hash.get((10, 10), 42);
        assert!(result.is_some());
        
        let result = hash.get((10, 10), 99);
        assert!(result.is_none());
    }

    #[test]
    fn test_update_position_nonexistent_element() {
        let mut hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (0, 0), (31, 31), 3);
        
        let result = hash.update_pos(42, (10, 10), (20, 20));
        assert!(matches!(result, MoveDelta::NoMove));

        let mut count = 0;
        result.for_each_with(&hash, |_item| {
            count += 1;
        });
        
        assert_eq!(count, 0);
    }

    #[test] 
    fn test_move_delta_no_move() {
        let mut hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (0, 0), (31, 31), 1);
        let player_data = create_player_data();
        
        // Insert elements in cells that will be adjacent to the new position
        hash.insert(1, player_data.clone(), (10, 10));   // Cell (1,1) - should be adjacent to (1,1)
        hash.insert(2, player_data.clone(), (20, 20));   // Cell (2,2) - should be adjacent to (1,1)  
        hash.insert(3, player_data, (5, 5));     // Cell (0,0) - should be adjacent to (1,1)
        
        // Move from (10, 10) to (15, 15) - same cell
        let result = hash.update_pos(1, (10, 10), (15, 15));
        assert!(matches!(result, MoveDelta::NoMove));
    }

    #[test]
    fn test_move_delta_some_move() {
        let mut hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (0, 0), (31, 31), 3);
        let player_data = create_player_data();
        
        // Insert elements in cells that will be adjacent to the new position
        hash.insert(1, player_data.clone(), (10, 10));   // Cell (1,1) 
        hash.insert(2, player_data.clone(), (20, 20));   // Cell (2,2) 
        hash.insert(3, player_data, (5, 5));     // Cell (0,0)
        
        // Move from (10, 10) to (25, 25) - different cell (3,3)
        let delta = hash.update_pos(1, (10, 10), (25, 25));
        
        // Should return a Delta since it moved to a different cell
        match delta {
            MoveDelta::Delta { from, to, check_radius, exclude_id } => {
                assert_eq!(from, (1, 1));
                assert_eq!(to, (3, 3));
                assert_eq!(check_radius, 3);
                assert_eq!(exclude_id, 1);
            }
            MoveDelta::NoMove => panic!("Expected Delta, got NoMove")
        }
    }

    #[test]
    fn test_move_delta_for_each_with_adjacent_elements() {
        let mut hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (0, 0), (31, 31), 1);
        let player_data = create_player_data();
        
        hash.insert(1, player_data.clone(), (5, 5));   // Cell (0,0)
        hash.insert(2, player_data.clone(), (20, 20));   // Cell (2,2)
        hash.insert(3, player_data, (10, 10));     // Cell (1,1)
        
        // Move from (5, 5) to (16, 16) - to different cell (2,2)
        let delta = hash.update_pos(1, (5, 5), (16, 16));
        
        let mut found_ids = Vec::new();
        delta.for_each_with(&hash, |item| {
            found_ids.push(item.0);
        });

        println!("{:?}", found_ids);
        
        // Should find only the elements that are now within adjacency range of (2,2) but weren't before
        // That is - id 2
        assert_eq!(found_ids.len(), 1);
        assert_eq!(found_ids[0], 2);
    }

    #[test]
    fn test_move_delta_for_each_wide_check_radius() {
        let mut hash = SpatialHash::<i32, Rc<RefCell<PlayerData>>>::new(8, (0, 0), (60, 60), 3);
        let player_data = create_player_data();
        
        // Insert elements that are NOT in any adjacent cell to new position 
        hash.insert(1, player_data.clone(), (10, 10));   // Cell (1,1)
        hash.insert(2, player_data.clone(), (50, 50));   // Cell (6,6)
        hash.insert(3, player_data.clone(), (32, 32));   // Cell (4,4)
        hash.insert(4, player_data, (40, 40));   // Cell (5,5)
        
        // Move from (10, 10) to (25, 25) - different cell (3,3)
        let delta = hash.update_pos(1, (10, 10), (25, 25));
        
        let mut found_ids = Vec::new();
        
        delta.for_each_with(&hash, |item| {
            found_ids.push(item.0);
        });
        
        // Elements with ids 2 and 4 should enter the adjacency
        // We do not know the order they will appear though.
        assert_eq!(found_ids.len(), 2);
        assert!(found_ids.contains(&2));
        assert!(found_ids.contains(&4));
    }
}
