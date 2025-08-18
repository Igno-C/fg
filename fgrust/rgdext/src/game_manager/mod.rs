use std::{cell::RefCell, collections::{BTreeMap, HashMap}, rc::Rc};

use godot::prelude::*;
use rgdext_shared::{genericevent::GenericPlayerEvent, playerdata::PlayerData};

use crate::eventqueue::{EQueue, GameEvent, ServerEvent};
use instance::{Instance};
use misc::*;

mod instance;
mod misc;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct GameManager {
    equeue: EQueue,

    /// net_id -> player's current instance pointer
    // player_instances: HashMap<i32, Gd<instance::Instance>>,
    player_locations: HashMap<i32, Gd<instance::Instance>>,
    /// pid -> pdata
    player_datas: HashMap<i32, PlayerDataEntry>,
    // current_players: i32,

    /// mapname -> list of instances
    instances: HashMap<String, Vec<Gd<instance::Instance>>>,

    /// pid -> Vec<net_id>
    /// 
    /// Holds the pid to net_id mapping for players whose data we're still waiting on
    datagets: BTreeMap<i32, Vec<i32>>,
    /// pid -> net_id
    full_datagets: BTreeMap<i32, i32>,

    base: Base<Node>
}

#[godot_api]
impl INode for GameManager {
    fn init(base: Base<Node>) -> Self {
        Self {
            equeue: EQueue::default(),

            player_locations: HashMap::new(),
            player_datas: HashMap::new(),
            instances: HashMap::new(),

            datagets: BTreeMap::new(),
            full_datagets: BTreeMap::new(),

            base
        }
    }

    fn ready(&mut self) {
        let mut db_server: Gd<Node> = self.base().get_node_as("/root/DbServer");
        db_server.connect("retrieved", &Callable::from_object_method(&self.to_gd(), "_on_db_retrieved"));
        db_server.connect("request_save", &Callable::from_object_method(&self.to_gd(), "_on_save_request"));
        self.base_mut().connect("save", &Callable::from_object_method(&db_server, "save"));
        self.base_mut().connect("retrieve", &Callable::from_object_method(&db_server, "retrieve"));

        godot_print!("Game manager node ready.\n");
    }

    fn process(&mut self, delta: f64) {
        let mut saves = Vec::with_capacity(10);
        self.player_datas.retain(|pid, pdata| {
            match pdata.tick(delta) {
                DataTickResult::Idle => true,
                DataTickResult::Save => {
                    saves.push(*pid);
                    true
                },
                DataTickResult::Timeout => false,
            }
        });

        for pid_to_save in saves {
            godot_print!("Saving data for pid {pid_to_save}");
            self.save_data(pid_to_save, false);
        }

        for e in self.equeue.iter_game() {
            match e {
                GameEvent::PlayerMove{x, y, speed, net_id} => self.player_move(x, y, speed, net_id),
                GameEvent::PlayerJoined{net_id, pid} => self.player_joined(net_id, pid),
                GameEvent::PlayerDisconnected{net_id} => self.player_despawn(net_id),
                GameEvent::PlayerJoinInstance{mapname, x, y, net_id} => self.player_join_instance(&mapname, x, y, net_id),
                // GameEvent::PlayerInteract{x, y, net_id} => self.player_interact(x, y, net_id),
                GameEvent::GenericEvent{event, net_id} => self.handle_generic_event(event, net_id),
                GameEvent::PDataRequest{pid, net_id} => self.player_retrieve_data(net_id, pid),
            }
        }
    }
}

#[godot_api]
impl GameManager {
    #[signal]
    fn retrieve(pid: i32, lock: bool);

    #[signal]
    fn save(pid: i32, data: PackedByteArray, unlock: bool);

    pub fn set_equeue(&mut self, e: EQueue) {
        self.equeue = e;
    }

    #[func]
    fn _on_save_request(&mut self, pid: i32) {
        self.save_data(pid, false);
    }

    #[func]
    fn _on_db_retrieved(&mut self, pid: i32, data: PackedByteArray) {
        let data = PlayerData::from_bytes(data.as_slice()).unwrap();

        if let Some(dataget) = self.datagets.remove(&pid) {
            let minimal_data = data.get_minimal().to_bytearray();
            
            for net_id in dataget {
                // Safe to clone PackedByteArray because it's CoW
                self.equeue.push_server(ServerEvent::PlayerDataResponse{data: minimal_data.clone(), net_id: net_id});
            }
        }

        let new_entry = if let Some(net_id) = self.full_datagets.remove(&pid) {
            // This is where the player truly joins the server and the Player object is created
            self.equeue.push_server(ServerEvent::PlayerDataResponse{data: data.to_bytearray(), net_id: net_id});
            
            let pdata = Rc::new(RefCell::new(data));
            let mut instance = self.get_instance(&pdata.borrow().location);
            instance.bind_mut().spawn_player(pdata.clone(), net_id);
            self.player_locations.insert(net_id, instance);
            PlayerDataEntry::new_with_id(pdata, net_id)
        }
        else {
            // Otherwise, hold the entry until it times out
            let pdata = Rc::new(RefCell::new(data));

            PlayerDataEntry::new(pdata)
        };

        // Store the data entry
        if let Some(pdata) = self.player_datas.get_mut(&pid) {
            *pdata = new_entry;
        }
        else {
            self.player_datas.insert(pid, new_entry);
        };
    }

    fn save_data(&mut self, pid: i32, unlock: bool) {
        if let Some(pdata) = self.player_datas.get(&pid) {
            let data = pdata.data.borrow().to_bytearray();
            self.signals().save().emit(pid, &data, unlock);
        }
        else {
            godot_error!("Tried to save pid {pid} while it's missing data");
        }
    }

    fn player_retrieve_data(&mut self, net_id: i32, pid: i32) {
        if let Some(pdata) = self.player_datas.get(&pid) {
            let data = pdata.data.borrow().get_minimal().to_bytearray();
            self.equeue.push_server(ServerEvent::PlayerDataResponse{data, net_id: net_id});
        }
        else {
            // If already waiting for the data from database, net_id to the waiting list
            // Otherwise, create new waiting list and do a new retrieval
            if let Some(waitlist) = self.datagets.get_mut(&pid) {
                waitlist.push(net_id);
            }
            else {
                self.datagets.insert(pid, vec![net_id]);
                self.signals().retrieve().emit(pid, false);
            }
        }
    }

    // Opens a new instance with the given mapname, adds it to instance repository, returns a pointer to it
    fn start_instance(&mut self, mapname: &str) -> Gd<Instance> {
        let mut inst = instance::Instance::new(mapname, self.equeue.clone());
        inst.bind_mut().mapname = mapname.to_string();
        
        // Map is loaded on node's _ready
        self.base_mut().add_child(&inst);

        // Append to list of instances
        if let Some(instances) = self.instances.get_mut(mapname) {
            instances.push(inst.clone());
        }
        // Or insert new vec with one instance
        else {
            self.instances.insert(mapname.to_string(), vec![inst.clone()]);
        }

        return inst;
    }

    /// Directly tied to GameEvent::PlayerMove
    fn player_move(&mut self, x: i32, y: i32, speed: i32, net_id: i32) {
        if let Some(i) = self.player_locations.get_mut(&net_id) {
            i.bind_mut().player_move(x, y, speed, net_id);
        }
    }

    // PlayerData is needed in here, it's passed to the instance
    // Sets up pointer to player's instance, rest is handled by the instance itself
    fn player_joined(&mut self, net_id: i32, pid: i32) {
        // A player with this pid is already on the server
        if self.player_datas.get(&pid).is_some_and(|e| e.net_id().is_some()) {
            self.equeue.push_server(ServerEvent::PlayerForceDisconnect{net_id});
            return;
        }

        self.full_datagets.insert(pid, net_id);

        self.signals().retrieve().emit(pid, true);
    }

    fn player_despawn(&mut self, net_id: i32) {
        if let Some(instance) = self.player_locations.get_mut(&net_id) {
            let data = instance.bind_mut().despawn_player(net_id);
            self.signals().save().emit(data.borrow().pid, &data.borrow().to_bytearray(), true);
        }
        self.player_locations.remove(&net_id);
    }

    /// Gets the best instance for given map name.
    /// 
    /// Will open a new instance if needed.
    fn get_instance(&mut self, mapname: &str) -> Gd<Instance> {
        if let Some(instances) = self.instances.get(mapname) {
            // Here is where checks whether an instance should be joinable should happen
            // If no instances are valid, we start a new one
            for instance in instances {
                // This will just return the first open instance for now
                return instance.clone();
            }
        }
        
        return self.start_instance(mapname);
    }

    // Requires the player to already be in an instance
    fn player_join_instance(&mut self, mapname: &str, x: i32, y: i32, net_id: i32) {
        let mut new_instance = self.get_instance(mapname);
        
        self.player_locations.entry(net_id).and_modify(|old_instance| {
            let data = old_instance.bind_mut().despawn_player(net_id);
            let mut b = data.borrow_mut();
            b.x = x; b.y = y;
            drop(b);
            new_instance.bind_mut().spawn_player(data, net_id);
            *old_instance = new_instance;

        });
        // self.player_locations.insert(net_id, instance.clone());

        // new_instance.bind_mut().spawn_player(data, x, y, net_id);
    }

    fn handle_generic_event(&mut self, event: GenericPlayerEvent, net_id: i32) {
        match event {
            GenericPlayerEvent::Interaction{x, y} => {
                if let Some(instance) = self.player_locations.get_mut(&net_id) {
                    instance.bind_mut().handle_interaction(x, y, net_id);
                }
            },
            GenericPlayerEvent::Err => {},
        }
    }
}
