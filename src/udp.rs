use embassy_net::udp::{PacketMetadata, UdpSocket};

use crate::control::{COMMANDS, Command, Mode};

#[embassy_executor::task]
pub(crate) async fn udp_task(stack: embassy_net::Stack<'static>) {
    stack.wait_config_up().await;

    let mut rx_meta = [PacketMetadata::EMPTY; 2];
    let mut tx_meta = [PacketMetadata::EMPTY; 1];
    let mut rx_buffer = [0u8; 128];
    let mut tx_buffer = [0u8; 64];

    let mut message = [0u8; 64];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    socket.bind(4000).unwrap();

    loop {
        let Ok((len, peer)) = socket.recv_from(&mut message).await else {
            continue;
        };

        let Ok(text) = core::str::from_utf8(&message[..len]) else {
            continue;
        };

        let text = text.trim();

        let mode = match text {
            "heartbeat" => Some(Mode::Heartbeat),
            "slow" => Some(Mode::Slow),
            "breathe" => Some(Mode::Breathe),
            "on" => Some(Mode::On),
            "off" => Some(Mode::Off),
            _ => None,
        };

        let reply: &[u8] = if text == "ping" {
            b"pong\n"
        } else if let Some(mode) = mode {
            COMMANDS.send(Command::Set(mode)).await;
            b"OK\n"
        } else {
            b"ERR: Options are `heartbeat`, `slow`, `on`, `off`, `breathe`, `ping`\n"
        };

        let _ = socket.send_to(reply, peer).await;
    }
}
