use zmq::{Context, SocketType};

use tmq::{request, Result, Multipart};
use utils::{
    generate_tcp_address, msg, sync_echo,
};

mod utils;

#[tokio::test]
async fn single_message() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let requester = request(&ctx).connect(&address)?.finish()?;

    let echo = sync_echo(address, SocketType::REP, 1);

    let m1 = "Msg";
    let m2 = "Msg (contd.)";
    let mut message = Multipart::from(vec![msg(m1.as_bytes()), msg(m2.as_bytes())]);
    let reply_receiver = requester.send(&mut message).await?;
    if let Ok((multipart,_)) = reply_receiver.recv().await {
        let expected: Multipart = vec![msg(m1.as_bytes()), msg(m2.as_bytes())].into();
        assert_eq!(expected, multipart);
    } else {
        panic!("Reply is missing.");
    }

    echo.join().unwrap();

    Ok(())
}


#[tokio::test]
async fn request_hammer() -> Result<()> {
    let address = generate_tcp_address();
    let ctx = Context::new();
    let mut req_sock = request(&ctx).connect(&address)?.finish()?;

    let count = 1_000;
    let echo = sync_echo(address, SocketType::REP, count);

    for i in 0..count {
        let m1 = format!("Msg #{}", i);
        let m2 = format!("Msg #{} (contd.)", i);
        let mut message = Multipart::from(vec![msg(m1.as_bytes()), msg(m2.as_bytes())]);
        let reply_sock = req_sock.send(&mut message).await?;
        let (multipart, req) = reply_sock.recv().await?;
        req_sock = req;
        let expected: Multipart = vec![msg(m1.as_bytes()), msg(m2.as_bytes())].into();
        assert_eq!(expected, multipart);
    }

    echo.join().unwrap();

    Ok(())
}
