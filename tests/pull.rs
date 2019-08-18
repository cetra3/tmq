#![feature(async_await)]

use zmq::{Context, SocketType};

use tmq::{pull, Result};
use utils::{check_receive_multiparts, generate_tcp_addres, msg, receive_multipart_repeated,
            sync_send_multipart_repeated, sync_send_multiparts};

mod utils;

#[tokio::test]
async fn receive_single_message() -> Result<()>
{
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let sock = pull(&ctx).bind(&address)?.finish();

    let thread = sync_send_multiparts(address, SocketType::PUSH, vec!(
        vec!(msg(b"hello"), msg(b"world"))
    ));

    check_receive_multiparts(sock, vec!(
        vec!(msg(b"hello"), msg(b"world"))
    )).await?;

    thread.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn receive_multiple_messages() -> Result<()>
{
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let sock = pull(&ctx).bind(&address)?.finish();

    let thread = sync_send_multiparts(address, SocketType::PUSH, vec!(
        vec!(msg(b"hello"), msg(b"world")),
        vec!(msg(b"second"), msg(b"message"))
    ));

    check_receive_multiparts(sock, vec!(
        vec!(msg(b"hello"), msg(b"world")),
        vec!(msg(b"second"), msg(b"message"))
    )).await?;

    thread.join().unwrap();

    Ok(())
}

#[tokio::test]
async fn receive_hammer() -> Result<()>
{
    let address = generate_tcp_addres();
    let ctx = Context::new();
    let sock = pull(&ctx).bind(&address)?.finish();

    let count: u64 = 1_000_000;
    let thread = sync_send_multipart_repeated(address, SocketType::PUSH,
                                    vec!(vec!(1, 2, 3), vec!(4, 5, 6)),
                                    count
    );

    receive_multipart_repeated(sock, vec!(msg(&[1, 2, 3]), msg(&[4, 5, 6])), count).await?;

    thread.join().unwrap();

    Ok(())
}
