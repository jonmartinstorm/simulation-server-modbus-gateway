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
use watertank_simulation_server::utils::protocol::{Payload, ReturnMessage};
use watertank_simulation_server::utils::protocol;

use std::{thread, time};

#[tokio::main]
async fn main() {
    // Setup logging, run with RUST_LOG=debug to see log
    env_logger::init();

    // Address for gateway to connect to
    let gw_addr = "0.0.0.0:9977".to_string();

    // Address for the websocket to listen to
    let ws_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:7799".to_string());

    // Create a tank
    let tank = WaterTank {
        level: 1000.0,
        areal: 1000000.0,
        height: 2000.0,
        inflow: 20.0,
        inflow_mean: 20.0,
        inflow_stddev: 3.0,
        max_inflow: 40.0,
        outflow: 20.0,
        max_outflow: 40.0,
        set_level: 1000.0,
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
    
    // run forever, but with a small delay so we dont overheat our pc.
    let delay = time::Duration::from_millis(20);
    loop {
        thread::sleep(delay);
    }
}

async fn run_simulation(txout: watch::Sender<WaterTank>, mut rxin: broadcast::Receiver<Payload>, mut tank: WaterTank) {
    tokio::spawn(async move {
        debug!("Starting simulation");
        loop {
            // Wait so we dont run too fast
            sleep(Duration::from_millis(300)).await;

            // Get and update outflow control setpoint
            let payload = rxin.recv().await.unwrap();
            tank.outflow = (payload.outflow as f32 / 65535.0) as f32 * 40.0; // create helper function


            tank.update_inflow();
            tank.update_level(0.3);
            
            txout.send(tank).unwrap();
            debug!("Tank: {:?}", tank);
        }
    });
}

async fn listen_ws(listener: TcpListener) {
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            handle_ws(stream).await;
        }
    });
}

async fn handle_ws(stream: TcpStream) {
    tokio::spawn(async move {
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
    });
}

async fn listen_tcp(listener: TcpListener, rxout: watch::Receiver<WaterTank>, txin: broadcast::Sender<Payload>) {
    tokio::spawn(async move {
        while let Ok((stream, addr)) = listener.accept().await {
            debug!("New connection from {:?}", addr);
            handle_gw(stream, rxout.clone(), txin.clone()).await;
        }
    });
}

async fn handle_gw(mut stream: TcpStream, rxout: watch::Receiver<WaterTank>, txin: broadcast::Sender<Payload>) {
    
    tokio::spawn(async move {
        debug!("Handle new connection");

        // In a loop, read data from the socket and write the data back.
        loop {
            let tank = *rxout.borrow();
            let tank_level = protocol::convert_f32_to_mobdus_u16(0.0, tank.height, tank.level);
            let tank_inflow = protocol::convert_f32_to_mobdus_u16(0.0, tank.max_inflow, tank.inflow);

            let (mut reader, mut writer) = stream.split();

            // read header length
            let mut len = vec![0; 1];
            match reader.peek(&mut len).await.unwrap() {
                0 => {break},
                _ => {},
            };

            let header = protocol::read_header(len, &mut reader).await;

            // read payload
            let payload = protocol::read_payload(header, &mut reader).await;
            
            debug!("Payload {:?}", payload);

            txin.send(payload).unwrap();

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