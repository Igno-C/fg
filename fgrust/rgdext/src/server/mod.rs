use std::collections::BTreeMap;

use godot::{classes::ENetMultiplayerPeer, prelude::*};
use rgdext_shared::genericevent::GenericPlayerEvent;
use crate::eventqueue::{EQueue, GameEvent, ServerEvent};

const AUTHENTICATION_TIMEOUT: f64 = 5.;
const AUTH_TOKEN_TIMEOUT: f64 = 4.5;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct Server {
    equeue: EQueue,

    port: i32,
    max_players: i32,
    current_players: i32,

    /// (token as bytes, pid, token timeout)
    pending_tokens: Vec<(Vec<u8>, i32, f64)>,
    /// net_id -> pid
    pending_joins: BTreeMap<i32, i32>,

    tick: i32,

    base: Base<Node>
}

#[godot_api]
impl INode for Server {
    fn init(base: Base<Node>) -> Self {
        Self {
            equeue: EQueue::default(),

            port: -1,
            max_players: -1,
            current_players: 0,

            pending_tokens: Vec::new(),
            pending_joins: BTreeMap::new(),

            tick: 0,

            base
        }
    }

    fn ready(&mut self) {
        let mut multiplayer_peer = ENetMultiplayerPeer::new_gd();
        multiplayer_peer.create_server_ex(self.port).max_clients(self.max_players).done();

        let mut multiplayer = self.base().get_multiplayer().unwrap().cast::<godot::classes::SceneMultiplayer>();
        multiplayer.connect("peer_authenticating", &Callable::from_object_method(&self.to_gd(), "on_peer_authenticating"));
        multiplayer.set_auth_callback(&Callable::from_object_method(&self.to_gd(), "verify_token"));
        multiplayer.set_auth_timeout(AUTHENTICATION_TIMEOUT);
        multiplayer.set_server_relay_enabled(false);
        multiplayer.set_multiplayer_peer(&multiplayer_peer);

        multiplayer.connect("peer_connected", &Callable::from_object_method(&self.to_gd(), "peer_connected"));
        multiplayer.connect("peer_disconnected", &Callable::from_object_method(&self.to_gd(), "peer_disconnected"));

        godot_print!("Server node ready.");
    }

    fn process(&mut self, delta: f64) {
        self.tick += 1;

        self.pending_tokens.retain_mut(|pending_token| {
            pending_token.2 += delta;
            if pending_token.2 > AUTH_TOKEN_TIMEOUT {
                godot_print!("Token {:?} timed out", pending_token.0);
                return false;
            }
            return true;
        });

        // let mut equeue = self.get_queue_mut();
        let iter = self.equeue.iter_server();
        for e in iter {
            match e {
                ServerEvent::PlayerMoveResponse{x, y, speed, pid, data_version, net_id} => {
                    self.base_mut().rpc_id(net_id.into(), "pmove", vslice![x, y, speed, data_version, pid]);
                },
                ServerEvent::PlayerDataResponse{data, net_id} => {
                    self.base_mut().rpc_id(net_id.into(), "pdata", vslice![data]);
                },
                ServerEvent::PlayerForceDisconnect{net_id} => {
                    self.base().get_multiplayer().unwrap().get_multiplayer_peer().unwrap().disconnect_peer(net_id);
                },
                ServerEvent::PlayerChat{text, from, from_pid, is_dm, net_id} => {
                    self.base_mut().rpc_id(net_id.into(), "pchat", vslice![text, from, from_pid, is_dm]);
                }
                ServerEvent::GenericResponse{response, net_id} => {
                    // let data = response.to_bytearray();
                    self.base_mut().rpc_id(net_id.into(), "pevent", vslice![response]);
                }
                ServerEvent::EntityMoveResponse{x, y, speed, entity_id, data_version, net_id} => {
                    self.base_mut().rpc_id(net_id.into(), "emove", vslice![x, y, speed, data_version, entity_id]);
                },
                ServerEvent::EntityDataResponse{interactable, walkable, related_scene, data, entity_id, net_id} => {
                    self.base_mut().rpc_id(net_id.into(), "edata", vslice![interactable, walkable, related_scene, data, entity_id]);
                }
            }
        };
    }
}

#[godot_api]
impl Server {
    #[func]
    fn set_config(&mut self, port: i32, max_players: i32) {
        self.port = port;
        self.max_players = max_players;
    }

    pub fn set_equeue(&mut self, e: EQueue) {
        // godot_print!("Set equeue to: {}", e.to_string());
        self.equeue = e;
    }

    // #[func]
    // fn from_config(port: i32, max_players: i32) -> Gd<Self> {
    //     Gd::from_init_fn(|base| {
    //         Self {
    //             equeue: EQueue::default(),

    //             port,
    //             max_players,
    //             current_players: 0,

    //             pending_tokens: Vec::new(),

    //             tick: 0,

    //             base
    //         }
    //     })
    // }

    #[func]
    fn max_players(&self) -> i32 {
        self.max_players
    }

    #[func]
    fn current_players(&self) -> i32 {
        self.current_players
    }

    #[func]
    fn port(&self) -> i32 {
        self.port
    }


    #[func]
    fn register_token(&mut self, token: String, pid: i32) {
        let bytes = token.into_bytes();
        godot_print!("Registered token with bytes {:?}", bytes);
        self.pending_tokens.push((bytes, pid, 0.));
    }

    #[func]
    fn on_peer_authenticating(&mut self, net_id: i32) {
        godot_print!("authenticating: {net_id}");
        self.base().get_multiplayer().unwrap().cast::<godot::classes::SceneMultiplayer>()
            .send_auth(net_id, &GString::from("ok").to_ascii_buffer());
    }

    // This is the auth_callback
    #[func]
    fn verify_token(&mut self, net_id: i32, token: PackedByteArray) {
        godot_print!("Verifying token {} from {}", token, net_id);
        let bytes = token.as_slice();
        if let Some(matched_i) = self.pending_tokens.iter().position(|(t, _, _)| t.eq(bytes)) {
            godot_print!("succeeded");
            let pid = self.pending_tokens[matched_i].1;
            self.pending_tokens.remove(matched_i);
            self.base().get_multiplayer().unwrap().cast::<godot::classes::SceneMultiplayer>().complete_auth(net_id);

            self.pending_joins.insert(net_id, pid);
            
        }
        else {
            godot_print!("failed");
        }
    }

    #[func]
    fn peer_connected(&mut self, net_id: i32) {
        godot_print!("connected: {net_id}");
        self.current_players += 1;
        if let Some(pid) = self.pending_joins.remove(&net_id) {
            self.equeue.push_game(
                GameEvent::PlayerJoined{net_id, pid}
            );
        }
    }

    #[func]
    fn peer_disconnected(&mut self, net_id: i32) {
        {self.equeue.push_game(
            GameEvent::PlayerDisconnected{net_id}
        )};

        self.current_players -= 1;
        godot_print!("disconnected: {net_id}");
    }

    
    #[rpc(any_peer, unreliable_ordered, call_remote, channel=0)]
    fn pmove(&mut self, x: i32, y: i32, speed: i32) {
        let net_id = self.base().get_multiplayer().unwrap().get_remote_sender_id();
        self.equeue.push_game(
            GameEvent::PlayerMove{x, y, speed, net_id}
        );
    }

    
    #[rpc(any_peer, reliable, call_remote, channel=1)]
    fn pdata(&mut self, pid: i32) {
        let net_id = self.base().get_multiplayer().unwrap().get_remote_sender_id();
        self.equeue.push_game(
            GameEvent::PDataRequest{net_id, pid}
        );
    }

    #[rpc(any_peer, reliable, call_remote, channel=2)]
    fn pevent(&mut self, pevent_bytes: PackedByteArray) {
        let net_id = self.base().get_multiplayer().unwrap().get_remote_sender_id();
        let event = GenericPlayerEvent::from_bytes(pevent_bytes.as_slice());
        self.equeue.push_game(
            GameEvent::GenericEvent{net_id, event}
        );
    }

    /// If target_pid is -1, that means zone chat
    #[rpc(any_peer, reliable, call_remote, channel=3)]
    fn pchat(&mut self, mut text: GString, target_pid: i32) {
        let net_id = self.base().get_multiplayer().unwrap().get_remote_sender_id();
        if text.len() > 400 {
            text = text.left(400);
        }
        self.equeue.push_game(
            GameEvent::PlayerChat{text, target_pid, net_id}
        );
    }

    #[rpc(authority, reliable, call_remote, channel=4)]
    fn emove(&mut self) {

    }

    #[rpc(any_peer, reliable, call_remote, channel=4)]
    fn edata(&mut self, x: i32, y: i32, entity_id: i32) {
        let net_id = self.base().get_multiplayer().unwrap().get_remote_sender_id();
        self.equeue.push_game(
            GameEvent::EDataRequest{x, y, entity_id, net_id}
        );
    }
}
