# jsonrpc-client-http

HTTP transport implementation for the JSON-RPC 2.0 clients generated by
[`jsonrpc-client-core`](../jsonrpc_client_core/index.html).

Uses the async Tokio based version of Hyper to implement a JSON-RPC 2.0 compliant HTTP
transport.

## Reusing connections

Each [`HttpTransport`](struct.HttpTransport.html) instance is backed by exactly one Hyper
`Client` and all [`HttpHandle`s](struct.HttpHandle.html) created through the same
`HttpTransport` also point to that same `Client` instance.

By default Hyper `Client`s have keep-alive activated and open connections will be kept and
reused if more requests are sent to the same destination before the keep-alive timeout is
reached.

## TLS / HTTPS

TLS support is compiled if the "tls" feature is enabled (it is enabled by default).

When TLS support is compiled in the instances returned by
[`HttpTransport::new`](struct.HttpTransport.html#method.new) and
[`HttpTransport::shared`](struct.HttpTransport.html#method.shared) support both plaintext http
and https over TLS, backed by the `hyper_tls::HttpsConnector` connector.

## Examples

See the integration test in `tests/localhost.rs` for code that creates an actual HTTP server
with `jsonrpc_http_server`, and sends requests to it with this crate.

Here is a small example of how to use this crate together with `jsonrpc_core`:

```rust
#[macro_use] extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;

use jsonrpc_client_http::HttpTransport;

jsonrpc_client!(pub struct FizzBuzzClient {
    /// Returns the fizz-buzz string for the given number.
    pub fn fizz_buzz(&mut self, number: u64) -> RpcRequest<String>;
});

fn main() {
    let transport = HttpTransport::new().unwrap();
    let transport_handle = transport.handle("https://api.fizzbuzzexample.org/rpc/").unwrap();
    let mut client = FizzBuzzClient::new(transport_handle);
    let result1 = client.fizz_buzz(3).call().unwrap();
    let result2 = client.fizz_buzz(4).call().unwrap();
    let result3 = client.fizz_buzz(5).call().unwrap();

    // Should print "fizz 4 buzz" if the server implemented the service correctly
    println!("{} {} {}", result1, result2, result3);
}
```

License: MIT/Apache-2.0
