// Copyright 2017 Amagicom AB.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate futures;
#[macro_use]
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;

extern crate tokio_core;

extern crate jsonrpc_core;
extern crate jsonrpc_http_server;
#[macro_use]
extern crate jsonrpc_macros;

use futures::Future;
use futures::future::Either;
use jsonrpc_client_http::HttpTransport;
use jsonrpc_core::{Error, IoHandler};
use jsonrpc_http_server::ServerBuilder;
use std::time::Duration;
use tokio_core::reactor::{Core, Timeout};


// Generate server API trait. Actual implementation at bottom of file.
build_rpc_trait! {
    pub trait ServerApi {
        #[rpc(name = "to_upper")]
        fn to_upper(&self, String) -> Result<String, Error>;

        #[rpc(name = "sleep")]
        fn sleep(&self, u64) -> Result<(), Error>;
    }
}

// Generate client struct with same API as server.
jsonrpc_client!(pub struct TestClient {
    pub fn to_upper(&mut self, string: &str) -> RpcRequest<String>;
    pub fn sleep(&mut self, time: u64) -> RpcRequest<()>;
});


#[test]
fn localhost_ping_pong() {
    // Spawn a server hosting the `ServerApi` API.
    let (_server, uri) = spawn_server();
    println!("Testing towards server at {}", uri);

    // Create the Tokio Core event loop that will drive the RPC client and the async requests.
    let mut core = Core::new().unwrap();

    // Create the HTTP transport handle and create a RPC client with that handle.
    let transport = HttpTransport::shared(&core.handle())
        .unwrap()
        .handle(&uri)
        .unwrap();
    let mut client = TestClient::new(transport);

    // Just calling the method gives back a `RpcRequest`, which is a future
    // that can be used to execute the actual RPC call.
    let rpc_future1 = client.to_upper("MaKe me UppeRcase!!1");
    let rpc_future2 = client.to_upper("foobar");

    let joined_future = rpc_future1.join(rpc_future2);
    // Run the event loop with the two calls to make them actually run.
    let (result1, result2) = core.run(joined_future).unwrap();

    assert_eq!("MAKE ME UPPERCASE!!1", result1);
    assert_eq!("FOOBAR", result2);
}

#[test]
fn dropped_rpc_request_should_not_crash_transport() {
    let (_server, uri) = spawn_server();

    let mut core = Core::new().unwrap();
    let transport = HttpTransport::shared(&core.handle())
        .unwrap()
        .handle(&uri)
        .unwrap();
    let mut client = TestClient::new(transport);

    let rpc = client.sleep(5).map_err(|e| e.to_string());
    let timeout = Timeout::new(Duration::from_millis(100), &core.handle()).unwrap();
    match core.run(rpc.select2(timeout.map_err(|e| e.to_string()))) {
        Ok(Either::B(((), _rpc))) => (),
        _ => panic!("The timeout did not finish first"),
    }

    // Now, sending a second request should still work. This is a regression test catching a
    // previous error where a dropped `RpcRequest` would crash the future running on the event loop.
    match core.run(client.sleep(0)) {
        Ok(()) => (),
        _ => panic!("Sleep did not return as it should"),
    }
}


/// Simple struct that will implement the RPC API defined at the top of this file.
struct Server;

impl ServerApi for Server {
    fn to_upper(&self, s: String) -> Result<String, Error> {
        Ok(s.to_uppercase())
    }

    fn sleep(&self, time: u64) -> Result<(), Error> {
        println!("Sleeping on server");
        ::std::thread::sleep(Duration::from_secs(time));
        Ok(())
    }
}

fn spawn_server() -> (jsonrpc_http_server::Server, String) {
    let server = Server;
    let mut io = IoHandler::new();
    io.extend_with(server.to_delegate());

    let server = ServerBuilder::new(io)
        .start_http(&"127.0.0.1:0".parse().unwrap())
        .unwrap();
    let uri = format!("http://{}", server.address());
    (server, uri)
}
