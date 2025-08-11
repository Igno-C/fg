use std::{cell::RefCell, collections::HashMap, rc::Rc};

use godot::prelude::*;
use rgdext_shared::playerdata::PlayerData;

use crate::eventqueue::{EQueue, GameEvent, EQueueInitializer};
use instance::{Instance, player::Player};

mod instance;

#[derive(GodotClass)]
#[class(no_init, base=Node)]
pub struct GameManager {
    equeue: EQueue,

    /// net_id -> player's current instance pointer
    // player_instances: HashMap<i32, Gd<instance::Instance>>,
    player_locations: HashMap<i32, (Rc<RefCell<Player>>, Gd<instance::Instance>)>,
    current_players: i32,

    /// mapname -> list of instances
    instances: HashMap<String, Vec<Gd<instance::Instance>>>,

    base: Base<Node>
}

#[godot_api]
impl INode for GameManager {
    fn ready(&mut self) {
        let q = self.base().get_node_as::<EQueueInitializer>("/root/QueueNode");
        self.set_equeue(q.bind().shared_queue.clone());

        let mut db_server: Gd<Node> = self.base().get_node_as("/root/DbServer");
        db_server.connect("retrieved", &Callable::from_object_method(&self.to_gd(), "on_db_retrieved"));
        self.base_mut().connect("save", &Callable::from_object_method(&db_server, "save_data"));
        self.base_mut().connect("retrieve", &Callable::from_object_method(&db_server, "retrieve_data"));

        godot_print!("Game manager node ready.\n");
    }

    fn process(&mut self, _delta: f64) {
        // let mut equeue = self.equeue.as_mut().unwrap().clone();
        // let mut eq = equeue.bind_mut();
        for e in self.equeue.iter_game() {
            match e {
                GameEvent::PlayerMove(x, y, speed, net_id) => self.player_move(x, y, speed, net_id),
                GameEvent::PlayerJoined{net_id, pid} => self.player_joined(net_id),
                GameEvent::PlayerDisconnected(net_id) => self.player_despawn(net_id),
                GameEvent::PlayerJoinInstance(mapname, x, y, net_id) => self.player_join_instance(&mapname, x, y, net_id),
                GameEvent::PlayerInteract(x, y, net_id) => self.player_interact(x, y, net_id),
                GameEvent::NewPlayerData{pid, data} => {}
            }
        }
    }
}

#[godot_api]
impl GameManager {
    #[signal]
    fn retrieve(pid: i32, force_create: bool);

    #[signal]
    fn save(pid: i32, data: PackedByteArray);

    #[func]
    fn from_config() -> Gd<Self> {
        Gd::from_init_fn(|base| {
            Self {
                equeue: EQueue::default(),
                player_locations: HashMap::new(),
                current_players: 0,
                instances: HashMap::new(),
                base
            }
        })
    }

    pub fn set_equeue(&mut self, e: EQueue) {
        godot_print!("Set equeue to {}", e.to_string());
        self.equeue = e;
    }

    // #[func]
    // fn current_players(&self) -> i32 {
    //     self.current_players
    // }

    // #[func]
    // fn max_players(&self) -> i32 {
    //     self.max_players
    // }


    #[func]
    fn on_db_retrieved(&mut self, pid: i32, data: PackedByteArray) {
        let data = PlayerData::from_bytes(data.as_slice()).unwrap();
        self.equeue.push_game(GameEvent::NewPlayerData{pid, data});
    }

    fn retrieve(&mut self, pid: i32, force_create: bool) {
        self.signals().retrieve().emit(pid, force_create);
    }

    fn save(&mut self, pid: i32) {
        
    }

    fn start_instance(&mut self, mapname: &str) -> Gd<Instance> {
        let mut inst = instance::Instance::new(mapname, self.equeue.clone());
        // let mut i = inst.bind_mut();
        // i.mapname = mapname.to_string();
        // std::mem::drop(i);
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
        if let Some((player, _i)) = self.player_locations.get_mut(&net_id) {
            player.borrow_mut().set_next_move(x, y, speed);
        }
    }

    // PlayerData is needed in here, it's passed to the instance
    // Sets up pointer to player's instance, rest is handled by the instance itself
    fn player_joined(&mut self, net_id: i32) {
        // Currently just creating new player data and player position
        // In the future, this will be fetched first
        let mut data = PlayerData::default();
        data.name = net_id.to_string();
        // data.location = "map1".to_string();
        let x = 0; let y = 0;

        let player = Player::new_rc(data, x, y, "map1");

        // Spawn the player on the instance
        // self.connect_player_to_map(mapname, net_id);
        let mut instance = self.get_instance(&player.borrow().data.location);
        instance.bind_mut().spawn_player(player.clone(), x, y, net_id);
        self.player_locations.insert(net_id, (player, instance));
        self.current_players += 1;
    }

    fn player_despawn(&mut self, net_id: i32) {
        if let Some((_, instance)) = self.player_locations.get_mut(&net_id) {
            instance.bind_mut().despawn_player(net_id);
            self.player_locations.remove(&net_id);
        }

        self.current_players -= 1;
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
        
        self.player_locations.entry(net_id).and_modify(|(p, old_instance)| {
            old_instance.bind_mut().despawn_player(net_id);
            new_instance.bind_mut().spawn_player(p.clone(), x, y, net_id);
            *old_instance = new_instance;

        });
        // self.player_locations.insert(net_id, instance.clone());

        // new_instance.bind_mut().spawn_player(data, x, y, net_id);
    }

    fn player_interact(&mut self, x: i32, y: i32, net_id: i32) {
        if let Some((_, instance)) = self.player_locations.get_mut(&net_id) {
            instance.bind_mut().handle_interaction(x, y, net_id);
        }
    }
}
