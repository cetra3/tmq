#![feature(async_await)]

use zmq::{Context, SocketType};
use tmq::{pull, Result};

mod utils;
use utils::{send_multiparts, msg, assert_receive_all_multiparts, generate_tcp_addres};

#[tokio::test]
async fn receive_single_message() -> Result<()>
{
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let sock = pull(&ctx).bind(&address)?.finish();

    let _ = send_multiparts(address, SocketType::PUSH, vec!(
        vec!(msg(b"hello"), msg(b"world"))
    ));

    assert_receive_all_multiparts(sock, vec!(
        vec!(msg(b"hello"), msg(b"world"))
    )).await?;

    Ok(())
}

#[tokio::test]
async fn receive_multiple_messages() -> Result<()>
{
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let sock = pull(&ctx).bind(&address)?.finish();

    let _ = send_multiparts(address, SocketType::PUSH, vec!(
        vec!(msg(b"hello"), msg(b"world")),
        vec!(msg(b"second"), msg(b"message"))
    ));

    assert_receive_all_multiparts(sock, vec!(
        vec!(msg(b"hello"), msg(b"world")),
        vec!(msg(b"second"), msg(b"message"))
    )).await?;

    Ok(())
}
