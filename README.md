# TMQ - Rust ZeroMQ bindings for Tokio

This crate bridges Tokio and ZeroMQ to allow for ZeroMQ in the async world.

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/tmq.svg
[crates-url]: https://crates.io/crates/tmq
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://choosealicense.com/licenses/mit/
[docs-badge]: https://img.shields.io/docsrs/tmq.svg
[docs-url]: https://docs.rs/crate/tmq/latest

## Changelog

### 0.5.0 - Extra setters/getters & new rust edition

- Added setters for curve key encryption [#45](https://github.com/cetra3/tmq/pull/45)
- Remove Redundant Imports [#43](https://github.com/cetra3/tmq/pull/43)
- Add more socket options & bump edition [#46](https://github.com/cetra3/tmq/pull/46)


### 0.4.0 - Bump Deps

Bump Deps & Pin future in RequestSender [#39](https://github.com/cetra3/tmq/pull/39)

### 0.3.1 - Iter Mut for Multipart

Adds an `iter_mut()` method to `Multipart`

### 0.3.0 - Tokio 1.0 Support

0.3.0 adds support for tokio 1.0 thanks to [YushiOMOTE](https://github.com/YushiOMOTE)!

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



