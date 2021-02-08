use std::env;
use env_logger;
use log::debug;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{sleep, Duration};
use tokio::sync::{watch, broadcast};

use simulation_server_v3::utils::watertank::WaterTank;
use simulation_server_v3::utils::protocol::{Point, Message, Header};

#[tokio::main]
async fn main() {
    // Setup logging, run with RUST_LOG=debug to see log
    env_logger::init();
    debug!("Log test.");

    // get address as argument or set default
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:9977".to_string());

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
    let listener = TcpListener::bind(&addr).await.unwrap();
    debug!("Listening on: {}", addr);

    // Create a few channels to talk across threads
    let (txout, rxout) = watch::channel(tank);
    let (txin, rxin) = broadcast::channel(2);

    // Start and run the listener and simulation.
    let t = listen_tcp(listener, rxout.clone(), txin.clone());
    let r = run_simulation(txout, rxin, tank);
    r.await;
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

async fn listen_tcp(listener: TcpListener, rxout: watch::Receiver<WaterTank>, txin: broadcast::Sender<(u16, u16)>) {
    
    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        debug!("New connection from {:?}", addr);
        handle(stream, rxout.clone(), txin.clone()).await;
    }
}

async fn handle(mut stream: TcpStream, rxout: watch::Receiver<WaterTank>, txin: broadcast::Sender<(u16, u16)>) {
    
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
            let hardcoded = Message {
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