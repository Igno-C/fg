use godot::{classes::{ENetMultiplayerPeer, SceneMultiplayer}, global::Error, prelude::*};


enum ServerType {
    Server{port: i32, max_connections: i32},
    Client{port: i32, address: GString},
    None
}

impl ServerType {
    fn is_client(&self) -> bool {
        match self {
            ServerType::Client{port: _, address: _} => true,
            _ => false,
        }
    }
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct ServerConnector {
    name: String,
    target_name: String,
    token: Option<PackedByteArray>,
    server_type: ServerType,

    num_connected: i32,
    
    base: Base<Node>
}

#[godot_api]
impl INode for ServerConnector {
    fn init(base: Base<Node>) -> Self {
        Self {
            name: "unset_name".to_string(),
            target_name: "unset_target_name".to_string(),
            token: None,
            server_type: ServerType::None,

            num_connected: 0,
            
            base
        }
    }
}

#[godot_api]
impl ServerConnector {
    #[func]
    fn start_server(&mut self) {
        let mut mult = SceneMultiplayer::new_gd();

        if self.token.is_some() {
            mult.set_auth_callback(&Callable::from_object_method(&self.to_gd(), "_verify_token"));
            mult.connect("peer_authenticating",
                &Callable::from_object_method(&self.to_gd(), "_on_peer_authenticating")
            );
            mult.connect("peer_authentication_failed",
                &Callable::from_object_method(&self.to_gd(), "_on_peer_authentication_failed")
            );
        }
        mult.connect("peer_connected",
            &Callable::from_object_method(&self.to_gd(), "_on_peer_connected_msg")
        );
        mult.connect("peer_disconnected",
            &Callable::from_object_method(&self.to_gd(), "_on_peer_disconnected_msg")
        );

        self.base().get_tree().unwrap()
            .set_multiplayer_ex(&mult)
            .root_path(&self.base().get_path())
            .done();

        let mut peer = ENetMultiplayerPeer::new_gd();
        match &self.server_type {
            ServerType::Client {port, address} => {
                let err = peer.create_client(address, *port);
                if err != Error::OK {
                    godot_print!("Error starting {} client: {:?}", self.name, err);
                }
            },
            ServerType::Server {port, max_connections} => {
                let err = peer.create_server_ex(*port).max_clients(*max_connections).done();
                if err != Error::OK {
                    godot_print!("Error starting {} server: {:?}", self.name, err);
                }
            },
            ServerType::None => {godot_error!("Server type unset! Call set_server() or set_client() first!");}
        }
        
        mult.set_server_relay_enabled(false);
        mult.set_multiplayer_peer(&peer);

        match &self.server_type {
            ServerType::Client{port, address} => {
                godot_print!("Started {} client at port {} and address {}", self.name, port, address);
            },
            ServerType::Server{port, max_connections: _} => {
                godot_print!("Started {} server at port {}", self.name, port);
            },
            ServerType::None => {}
        }
        if self.token.is_some() {
            godot_print!("Using token verification")
        }
    }

    #[func]
    fn is_connected(&self) -> bool {
        self.num_connected > 0
    }

    #[func]
    fn num_connected(&self) -> i32 {
        self.num_connected
    }

    #[func]
    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    #[func]
    fn set_target_name(&mut self, name: String) {
        self.target_name = name;
    }

    #[func]
    fn set_client(&mut self, port: i32, address: GString) {
        self.server_type = ServerType::Client{port, address};
    }

    #[func]
    fn set_server(&mut self, port: i32, max_connections: i32) {
        self.server_type = ServerType::Server{port, max_connections};
    }

    #[func]
    fn set_token(&mut self, token: GString) {
        self.token = Some(token.to_ascii_buffer());
    }

    #[func]
    fn _verify_token(&self, net_id: i32, data: PackedByteArray) {
        let mut mult = self.base().get_multiplayer().unwrap().cast::<SceneMultiplayer>();
        match &self.server_type {
            ServerType::Client{port: _, address: _} => {
                godot_print!("Received authentication from {}: {}", self.target_name, data);
                mult.complete_auth(net_id);
            },
            ServerType::Server{port: _, max_connections: _} => {
                godot_print!("Authenticating token for {} {}", self.target_name, net_id);
                if let Some(token) = &self.token {
                    if token.eq(&data) {
                        godot_print!("Token matched for {} {}", self.target_name, net_id);
                        let err = mult.complete_auth(net_id);
                        if err != Error::OK {
                            godot_error!("Error completing auth: {:?}", err)
                        }
                    }
                    else {
                        godot_print!("Token invalid for {} {}", self.target_name, net_id);
                    }
                }
                else {
                    godot_error!("Somehow token unset while auth enabled!");
                }
            },
            ServerType::None => {}
        }
    }

    #[func]
    fn _on_peer_authenticating(&self, net_id: i32) {
        godot_print!("Authenticating with {} {}", self.target_name, net_id);
        if self.server_type.is_client() {
            self.base().get_multiplayer().unwrap().cast::<SceneMultiplayer>().send_auth(net_id, self.token.as_ref().unwrap());
        }
        else {
            let bytes = GString::from("ok").to_ascii_buffer();
            self.base().get_multiplayer().unwrap().cast::<SceneMultiplayer>().send_auth(net_id, &bytes);
        }
    }

    #[func]
    fn _on_peer_authentication_failed(&self, net_id: i32) {
        godot_print!("Failed authentication with {} {}", self.target_name, net_id);
    }

    #[func]
    fn _on_peer_connected_msg(&mut self, net_id: i32) {
        self.num_connected += 1;
        match &self.server_type {
            ServerType::Client{port: _, address: _} => {
                let net_id = self.base().get_multiplayer().unwrap().get_unique_id();
                godot_print!("Connected to {} as {}", self.target_name, net_id);
            },
            ServerType::Server{port: _, max_connections: _} => {
                godot_print!("Client {} connected to {}", net_id, self.name);
            },
            ServerType::None => {}
        }
    }

    #[func]
    fn _on_peer_disconnected_msg(&mut self, net_id: i32) {
        self.num_connected -= 1;
        match &self.server_type {
            ServerType::Client{port: _, address: _} => {
                let net_id = self.base().get_multiplayer().unwrap().get_unique_id();
                godot_print!("Disonnected from {} as {}", self.target_name, net_id);
            },
            ServerType::Server{port: _, max_connections: _} => {
                godot_print!("Client {} disconnected from {}", net_id, self.name);
            },
            ServerType::None => {}
        }
    }
}


