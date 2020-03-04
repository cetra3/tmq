# TMQ - Rust ZeroMQ bindings for Tokio

This crate bridges Tokio and ZeroMQ to allow for ZeroMQ in the async world.

## Currently Implemented Sockets

* Request/Reply
* Publish/Subscribe
* Dealer/Router
* Push/Pull

## Examples

Please see the `examples` directory for a full set of examples.  They are paired together based upon the socket types.

## Usage

Usage is made to be simple, but opinionated.   See the examples for working code, but in general, you need to import `tokio` and `tmq::*`

### Publish

To publish messages to all connected subscribers, you can use the `publish` function:

```rust
use tmq::{publish, Context, Result};

use futures::SinkExt;
use log::info;
use std::env;
use std::time::Duration;
use tokio::time::delay_for;

#[tokio::main]
async fn main() -> Result<()> {

    let mut socket = publish(&Context::new()).bind("tcp://127.0.0.1:7899")?;

    let mut i = 0;

    loop {
        i += 1;

        socket
            .send(vec!["topic", &format!("Broadcast #{}", i)])
            .await?;

        delay_for(Duration::from_secs(1)).await;
    }
}
```

### Subscribe

a subscribe socket is a `Stream` that reads in values from a publish socket.  You specify the filter prefix using the `subscribe` method, using `""` for all messages.

```rust
use futures::StreamExt;

use tmq::{subscribe, Context, Result};

use std::env;

#[tokio::main]
async fn main() -> Result<()> {

    let mut socket = subscribe(&Context::new())
        .connect("tcp://127.0.0.1:7899")?
        .subscribe(b"topic")?;

    while let Some(msg) = socket.next().await {
        println!(
            "Subscribe: {:?}",
            msg?.iter()
                .map(|item| item.as_str().unwrap_or("invalid text"))
                .collect::<Vec<&str>>()
        );
    }
    Ok(())
}
```



