use std::env;
use env_logger;
use log::debug;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{sleep, Duration};
use tokio::sync::{watch, broadcast};

use futures_util::StreamExt;
use futures::sink::SinkExt;
use tokio_tungstenite::tungstenite::Message;

use watertank_simulation_server::utils::watertank::WaterTank;
use watertank_simulation_server::utils::protocol::{Point, ReturnMessage, Header};

#[tokio::main]
async fn main() {
    // Setup logging, run with RUST_LOG=debug to see log
    env_logger::init();
    debug!("Log test.");

    // Address for gateway to connect to
    let gw_addr = "0.0.0.0:9977".to_string();

    // Address for the websocket to listen to
    let ws_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:7799".to_string());

    // Create a tank
    let tank = WaterTank {
        level: 1000,
        areal: 1000000,
        height: 2000,
        inflow: 20.0,
        inflow_mean: 20.0,
        inflow_stddev: 3.0,
        outflow: 20.0,
        set_level: 1000,
    };

    // Setup tcplistener to the gateway
    let gw_listener = TcpListener::bind(&gw_addr).await.unwrap();
    debug!("GW listening on: {}", gw_addr);

    // Setup tcplistener for the websocket
    let ws_listener = TcpListener::bind(&ws_addr).await.unwrap();
    debug!("WS listening on: {}", ws_addr);

    // Create a few channels to talk across threads
    let (txout, rxout) = watch::channel(tank);
    let (txin, rxin) = broadcast::channel(2);

    // Start and run the listeners and simulation.
    let t = listen_tcp(gw_listener, rxout.clone(), txin.clone());
    let ws = listen_ws(ws_listener);
    let r = run_simulation(txout, rxin, tank);
    r.await;
    ws.await;
    t.await;
}

async fn run_simulation(txout: watch::Sender<WaterTank>, mut rxin: broadcast::Receiver<(u16, u16)>, mut tank: WaterTank) {
    tokio::spawn(async move {
        loop {
            // Wait so we dont run too fast
            sleep(Duration::from_millis(300)).await;

            // Get and update outflow control setpoint
            let (outflow, _r) = rxin.recv().await.unwrap();
            tank.outflow = (outflow as f32 / 65535.0) as f64 * 40.0;


            tank.update_inflow();
            tank.update_level(0.3);
            
            txout.send(tank).unwrap();
            debug!("Tank: {:?}", tank);
        }
    });
}

async fn listen_ws(listener: TcpListener) {
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_ws(stream));
    }
}

async fn handle_ws(stream: TcpStream) {
    let addr = stream.peer_addr().expect("connected streams should have a peer address");
    debug!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    debug!("New WebSocket connection: {}", addr);

    let (mut write, _) = ws_stream.split();
    let mut i = 0;

    loop {
        sleep(Duration::from_millis(100)).await;
        i += 1;
        let message = Message::Text(i.to_string());
        write.send(message).await.unwrap();
    }
    //read.forward(write).await.expect("Failed to forward message")
}

async fn listen_tcp(listener: TcpListener, rxout: watch::Receiver<WaterTank>, txin: broadcast::Sender<(u16, u16)>) {
    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        debug!("New connection from {:?}", addr);
        handle_gw(stream, rxout.clone(), txin.clone()).await;
    }
}

async fn handle_gw(mut stream: TcpStream, rxout: watch::Receiver<WaterTank>, txin: broadcast::Sender<(u16, u16)>) {
    
    tokio::spawn(async move {
        debug!("Handle new connection");

        // In a loop, read data from the socket and write the data back.
        loop {
            let tank = *rxout.borrow();
            let max = 65535 as f32 / tank.height as f32;
            let tank_level = (tank.level as f32 * max) as u16;
            
            let max = 65535 as f32 / 40 as f32;
            let tank_inflow = (tank.inflow as f32 * max) as u16;

            let (mut reader, mut writer) = stream.split();

            // read header length
            let mut len = vec![0; 1];
            match reader.peek(&mut len).await.unwrap() {
                0 => {break},
                _ => {},
            };
            reader.read(&mut len).await.unwrap();

            // read header
            let mut header = vec![0; len[0] as usize];
            reader.read(&mut header).await.unwrap();
            let header_string = std::str::from_utf8(&header).unwrap();  
            let header: Header = serde_json::from_str(header_string).unwrap();

            // read payload
            let mut payload = vec![0; header.len as usize];
            reader.read(&mut payload).await.unwrap();
            let payload_string = std::str::from_utf8(&payload).unwrap();
            let payload: Point = serde_json::from_str(payload_string).unwrap();
            debug!("Payload {:?}", payload);

            txin.send((payload.x as u16, payload.y as u16)).unwrap();

            // write something random
            let hardcoded = ReturnMessage {
                msg_type: String::from("input-register"),
                address: 0,
                tank_level: tank_level,
                tank_inflow: tank_inflow,
            };
            let mut hardcoded = serde_json::to_string(&hardcoded).unwrap();
            debug!("Sending {}", hardcoded);
            hardcoded.push('\n');
            writer.write_all(hardcoded.as_bytes()).await.unwrap();
        }
    });
}