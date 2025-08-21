use godot::{classes::{multiplayer_api::RpcMode, multiplayer_peer::TransferMode, ENetMultiplayerPeer}, prelude::*};
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

    // pending_players: Vec<(i32, f64)>,
    /// (token as bytes, pid, token timeout)
    pending_tokens: Vec<(Vec<u8>, i32, f64)>,

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

            // pending_players: Vec::new(),
            pending_tokens: Vec::new(),

            tick: 0,

            base
        }
    }

    fn ready(&mut self) {
        // let q = self.base().get_node_as::<EQueueInitializer>("/root/QueueNode");
        // self.set_equeue(q.bind().shared_queue.clone());

        // Unreliable config for player movement
        let move_config: Dictionary = vdict! {
            "rpc_mode": RpcMode::ANY_PEER,
            "transfer_mode": TransferMode::UNRELIABLE_ORDERED,
            "call_local": false,
            "channel": 0
        };

        let data_config: Dictionary = vdict! {
            "rpc_mode": RpcMode::ANY_PEER,
            "transfer_mode": TransferMode::RELIABLE,
            "call_local": false,
            "channel": 1
        };

        let player_event_config: Dictionary = vdict! {
            "rpc_mode": RpcMode::ANY_PEER,
            "transfer_mode": TransferMode::RELIABLE,
            "call_local": false,
            "channel": 2
        };

        self.base_mut().rpc_config("pmove", &Variant::from(move_config));

        self.base_mut().rpc_config("pdata", &Variant::from(data_config));

        self.base_mut().rpc_config("pevent", &Variant::from(player_event_config));

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

        godot_print!("Server node ready.\n");
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
                ServerEvent::GenericResponse{response, net_id} => {
                    let data = response.to_bytearray();
                    self.base_mut().rpc_id(net_id.into(), "pevent", vslice![data]);
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

    #[func]
    fn from_config(port: i32, max_players: i32) -> Gd<Self> {
        Gd::from_init_fn(|base| {
            Self {
                equeue: EQueue::default(),

                port,
                max_players,
                current_players: 0,

                // pending_players: Vec::new(),
                pending_tokens: Vec::new(),

                tick: 0,

                base
            }
        })
    }

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

            self.equeue.push_game(
                GameEvent::PlayerJoined{net_id, pid}
            );
        }
        else {
            godot_print!("failed");
        }
    }

    #[func]
    fn peer_connected(&mut self, net_id: i32) {
        godot_print!("connected: {net_id}");
        self.current_players += 1;
    }

    #[func]
    fn peer_disconnected(&mut self, net_id: i32) {
        {self.equeue.push_game(
            GameEvent::PlayerDisconnected{net_id}
        )};

        self.current_players -= 1;
        godot_print!("disconnected: {net_id}");
    }

    // #[rpc(any_peer, unreliable_ordered, call_remote, channel=0)]
    #[func]
    fn pmove(&mut self, x: i32, y: i32, speed: i32) {
        let net_id = self.base().get_multiplayer().unwrap().get_remote_sender_id();
        self.equeue.push_game(
            GameEvent::PlayerMove{x, y, speed, net_id}
        );
    }

    #[func]
    fn pdata(&mut self, pid: i32) {
        let net_id = self.base().get_multiplayer().unwrap().get_remote_sender_id();
        self.equeue.push_game(
            GameEvent::PDataRequest{net_id, pid}
        );
    }

    #[func]
    fn pevent(&mut self, pevent_bytes: PackedByteArray) {
        let net_id = self.base().get_multiplayer().unwrap().get_remote_sender_id();
        let event = GenericPlayerEvent::from_bytes(pevent_bytes.as_slice());
        self.equeue.push_game(
            GameEvent::GenericEvent{net_id, event}
        );
    }

    /// If target_pid is -1, that means zone chat
    #[func]
    fn pchat(&mut self, text: GString, target_pid: i32) {
        let net_id = self.base().get_multiplayer().unwrap().get_remote_sender_id();
        self.equeue.push_game(
            GameEvent::PlayerChat{text, target_pid, net_id}
        );
    }
}
