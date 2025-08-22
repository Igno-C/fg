use std::{cell::RefCell, collections::{BTreeMap, HashMap}, rc::Rc, sync::{atomic::{AtomicBool, Ordering}, Arc}};

use godot::prelude::*;
use rgdext_shared::{genericevent::GenericPlayerEvent, playerdata::PlayerData};

use crate::eventqueue::{EQueue, GameEvent, ServerEvent};
use instance::{player::Player, Instance};
use misc::*;

mod instance;
mod misc;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct GameManager {
    server_name: String,
    equeue: EQueue,

    /// net_id -> player's current instance pointer
    // player_instances: HashMap<i32, Gd<instance::Instance>>,
    player_locations: HashMap<i32, Gd<instance::Instance>>,
    /// pid -> playerdataentry
    player_datas: HashMap<i32, PlayerDataEntry>,

    /// mapname -> list of instances
    instances: HashMap<String, Vec<Gd<instance::Instance>>>,

    /// pid -> Vec<net_id>
    /// 
    /// Holds the pid to net_id mapping for players whose data we're still waiting on
    datagets: BTreeMap<i32, Vec<i32>>,
    /// pid -> net_id
    full_datagets: BTreeMap<i32, i32>,

    /// Set to true on getting SIGINT
    got_sigint: Arc<AtomicBool>,

    base: Base<Node>
}

#[godot_api]
impl INode for GameManager {
    fn init(base: Base<Node>) -> Self {
        Self {
            server_name: String::new(),
            equeue: EQueue::default(),

            player_locations: HashMap::new(),
            player_datas: HashMap::new(),
            instances: HashMap::new(),

            datagets: BTreeMap::new(),
            full_datagets: BTreeMap::new(),

            got_sigint: Arc::new(AtomicBool::new(false)),

            base
        }
    }

    fn ready(&mut self) {
        // Setting up handler for SIGINT
        let got_sigint_clone = self.got_sigint.clone();
        ctrlc::set_handler(move || {
            got_sigint_clone.store(true, Ordering::Relaxed);
        }).unwrap();

        let mut db_server: Gd<Node> = self.base().get_node_as("/root/DbServer");
        db_server.connect("retrieved", &Callable::from_object_method(&self.to_gd(), "_on_db_retrieved"));
        db_server.connect("request_save", &Callable::from_object_method(&self.to_gd(), "_on_save_request"));
        self.base_mut().connect("save", &Callable::from_object_method(&db_server, "save"));
        self.base_mut().connect("retrieve", &Callable::from_object_method(&db_server, "retrieve"));

        godot_print!("Game manager node ready.\n");
    }

    fn process(&mut self, delta: f64) {
        // What happens on receiving SIGINT
        if self.got_sigint.load(Ordering::Relaxed) {
            if self.player_datas.is_empty() {
                godot_print!("Quitting now");
                self.base().get_tree().unwrap().quit();
                return;
            }

            godot_print!("Doing full save before quitting");
            self.full_save();
            // godot_print!("Waiting 5 seconds");

            // self.base().get_tree().unwrap()
            //     .create_timer(5.).unwrap()
            //     .signals().timeout()
            //     .connect_other(&self.to_gd(),
            //         |a| {
            //             godot_print!("Quitting now");
            //             a.base().get_tree().unwrap().quit();
            //         }
            // );
        }

        let mut saves = Vec::with_capacity(10);
        self.player_datas.retain(|pid, pdata| {
            match pdata.tick(delta) {
                DataTickResult::Idle => true,
                DataTickResult::Save => {
                    saves.push(*pid);
                    true
                },
                DataTickResult::Timeout => {
                    godot_print!("Data for pid {} timed out.", pid);
                    false
                },
            }
        });

        for pid_to_save in saves {
            self.save_dataentry(pid_to_save);
        }

        for e in self.equeue.iter_game() {
            match e {
                GameEvent::PlayerMove{x, y, speed, net_id} => self.player_move(x, y, speed, net_id),
                GameEvent::PlayerJoined{net_id, pid} => self.player_joined(net_id, pid),
                GameEvent::PlayerDisconnected{net_id} => self.player_despawn(net_id),
                GameEvent::PlayerJoinInstance{mapname, x, y, net_id} => self.player_join_instance(&mapname, x, y, net_id),
                GameEvent::PlayerChat{text, target_pid, net_id} => {self.broadcast_chat(text, target_pid, net_id)},
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
    fn set_server_name(&mut self, name: String) {
        self.server_name = name;
    }

    #[func]
    fn _on_save_request(&mut self, pid: i32) {
        self.save_dataentry(pid);
    }

    /// Does unlocking saves for all playerdatas, consuming them, then disables process for itself and all instances
    fn full_save(&mut self) {
        self.player_locations.clear();
        for instances in std::mem::take(&mut self.instances).into_values() {
            for mut instance in instances {
                instance.queue_free();
            }
        }
        
        for (pid, dataentry) in std::mem::take(&mut self.player_datas).into_iter() {
            if let PlayerDataEntry::ActivePlayer{player, net_id, age: _} = dataentry {
                let bytearray = player.borrow().data().to_bytearray();

                self.signals().save().emit(pid, &bytearray, true);
                self.equeue.push_server(ServerEvent::PlayerForceDisconnect{net_id});
            }
        }

        self.base_mut().set_process(false);
    }

    #[func]
    fn _on_db_retrieved(&mut self, pid: i32, data: PackedByteArray) {
        let data = PlayerData::from_bytes(data.as_slice());

        if let Some(dataget) = self.datagets.remove(&pid) {
            if let Ok(data) = &data {
                let minimal_data = data.get_minimal().to_bytearray();
            
                for net_id in dataget {
                    // Safe to clone PackedByteArray because it's CoW
                    self.equeue.push_server(ServerEvent::PlayerDataResponse{data: minimal_data.clone(), net_id: net_id});
                }
            }
            
        }

        // This is where the player truly joins the server and the Player object is created
        let new_entry = if let Some(net_id) = self.full_datagets.remove(&pid) {
            if let Ok(data) = data {
                self.equeue.push_server(ServerEvent::PlayerDataResponse{data: data.to_bytearray(), net_id: net_id});
            
                let mut instance = self.get_instance(&data.location);
                let player = Player::new_rc(data, &self.server_name);
                instance.bind_mut().spawn_player(player.clone(), net_id);
                self.player_locations.insert(net_id, instance);
    
                PlayerDataEntry::new_active(player, net_id)
            }
            else {
                self.equeue.push_server(ServerEvent::PlayerForceDisconnect{net_id});
                panic!("Received invalid data for pid {pid}");
            }
        }
        else {
            PlayerDataEntry::new_inactive(data.unwrap())
        };

        // Store the data entry
        if let Some(pdata) = self.player_datas.get_mut(&pid) {
            *pdata = new_entry;
        }
        else {
            self.player_datas.insert(pid, new_entry);
        };
    }

    /// Saves data stored in dataentry with given pid
    /// 
    /// Does not save dataentries that are not active players
    fn save_dataentry(&mut self, pid: i32) {
        if let Some(dataentry) = self.player_datas.get(&pid) {
            if let PlayerDataEntry::ActivePlayer{player, net_id: _, age: _} = dataentry {
                let data = player.borrow().data().to_bytearray();
                self.signals().save().emit(pid, &data, false);
            }
        }
        else {
            godot_error!("Tried to save pid {pid} while it's missing data");
        }
    }

    // Eats the player rc, resets server_name, saves with unlock
    fn save_unlocking(&mut self, player: Rc<RefCell<Player>>) {
        let mut b = player.borrow_mut();
        b.data_mut().server_name.clear();
        let pid = b.pid();
        self.signals().save().emit(pid, &b.data().to_bytearray(), true);
        drop(b); drop(player);
    }

    fn player_retrieve_data(&mut self, net_id: i32, pid: i32) {
        if let Some(pdataentry) = self.player_datas.get(&pid) {
            let data = match pdataentry {
                PlayerDataEntry::RawData{data, age: _} => data.to_bytearray(),
                PlayerDataEntry::ActivePlayer{player, net_id: _, age: _} => player.borrow().data().to_bytearray(),
            };
            self.equeue.push_server(ServerEvent::PlayerDataResponse{data, net_id});
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
        if self.player_datas.get(&pid).is_some_and(|e| e.is_active()) {
            self.equeue.push_server(ServerEvent::PlayerForceDisconnect{net_id});
            godot_print!("Double join attempt from pid {pid}");
            return;
        }

        self.full_datagets.insert(pid, net_id);

        self.signals().retrieve().emit(pid, true);
    }

    // Turns the player data into an inactive entry
    fn player_despawn(&mut self, net_id: i32) {
        if let Some(instance) = self.player_locations.get_mut(&net_id) {
            let player = instance.bind_mut().despawn_player(net_id);
            let pid = player.borrow().pid();
            self.save_unlocking(player);
            
            self.player_locations.remove(&net_id);
            if let Some(pdataentry) = self.player_datas.remove(&pid) {
                if let PlayerDataEntry::ActivePlayer{player, net_id: _, age: _} = pdataentry {
                    let data = match std::rc::Rc::try_unwrap(player) {
                        Ok(p) => p.into_inner().into_data(),
                        Err(rc) => panic!("Couldn't unwrap player rc for {net_id}! refcount: {}", std::rc::Rc::strong_count(&rc)),
                    };
                    self.player_datas.insert(pid, PlayerDataEntry::new_inactive(data));
                };
            };
        }
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
            let player = old_instance.bind_mut().despawn_player(net_id);
            let mut b = player.borrow_mut();
            b.set_full_pos(x, y, 0);
            drop(b);
            new_instance.bind_mut().spawn_player(player, net_id);
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

    fn broadcast_chat(&self, text: GString, target_pid: i32, net_id: i32) {

    }
}
