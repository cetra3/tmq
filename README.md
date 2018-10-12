# TMQ - Rust ZeroMQ bindings for Tokio

This crate bridges Tokio and ZeroMQ to allow for ZeroMQ in the async world.

Currently a WIP

## Currently Implemented Sockets

* Request
* Response
* Subscribe
* Publish

## Examples

There are two sets of examples, `publish/subscribe` and `request/response`.

Bring up two terminals and run either:

```sh
cargo run --example publish
# In Another Terminal
cargo run --example subcribe
```

Or:

```sh
cargo run --example request
# Another Terminal
cargo run --example response
```


## Usage

Usage is made to be simple, but opinionated.   See the examples for working code, but in general, you need to import `tokio` and `tmq::*`

### Request

A request is a `Stream` takes an input stream of Messages (using the `with` function), sends them to a response socket, and then returns the messages as a stream.

```rust
let request = request(&Context::new())
    .connect("tcp://127.0.0.1:7899")
    .expect("Couldn't connect")
    .with(stream::iter_ok(vec!["Message1", "Message2", "Message3"].into_iter().map(|val| Message::from(val))))
    .for_each(|val| {
        info!("Response: {}", val.as_str().unwrap_or(""));
        Ok(())
    }).map_err(|err| {
        error!("Error with request: {}", err);
    });

tokio::run(request);
```

### Response

A response socket is a `Future` that receives messages, responds to them, and sends them back as per the `Responder` implementation:

```rust
let responder = respond(&Context::new())
    .bind("tcp://127.0.0.1:7899")
    .expect("Couldn't bind address")
    .with(|msg: Message| {
        info!("Request: {}", msg.as_str().unwrap_or(""));
        Ok(msg)
    }).map_err(|err| {
        error!("Error from server:{}", err);
    });

tokio::run(responder);
```

#### Responder trait

The `with` function takes anything that implements the `Responder` trait or a closure as above:

```rust
//You can use a struct to respond by implementing the `Responder` trait
pub struct EchoResponder {}

impl Responder for EchoResponder {
    type Output = FutureResult<zmq::Message, Error>;

    fn respond(&mut self, msg: zmq::Message) -> Self::Output {
        return Ok(msg).into();
    }
}

//Or you can use a free-floating function
fn echo(msg: zmq::Message) -> impl Future<Item = zmq::Message, Error = Error> {
    return ok(msg);
}
```

### Publish

A publish socket is a `Sink` that takes values, and sends them to any subscribe sockets connected (note: ZeroMQ will drop messages if noone is connected).

```rust
let mut i = 0;

let broadcast = Interval::new_interval(Duration::from_millis(1000))
    .map(move |_| {
        i += 1;
        let message = format!("Broadcast #{}", i);
        Message::from(&message)
    });

let request = publish(&Context::new())
    .bind("tcp://127.0.0.1:7899")
    .expect("Couldn't bind")
    .finish()
    .send_all(broadcast)
    .map(|_| ())
    .map_err(|e| {
        error!("Error publishing:{}", e);
    });

tokio::run(request);
```

### Subscribe

a subscribe socket is a `Stream` that reads in values from a publish socket.  You specify the filter prefix using the `subscribe` method, using `""` for all messages.

```rust
let request = subscribe(&Context::new())
    .connect("tcp://127.0.0.1:7899")
    .expect("Couldn't connect")
    .subscribe("")
    .for_each(|val| {
        info!("Subscribe: {}", val.as_str().unwrap_or(""));
        Ok(())
    }).map_err(|e| {
        error!("Error Subscribing: {}", e);
    });
```



