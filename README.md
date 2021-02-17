# simulation-server-modbus-gateway
A simulation server for a water tank with a modbus gateway for control

- The server is written in rust by me
- The gateway is written in python3 using pymodbus and is a heavily modified version of one of the pymodbus examples

## How to run
### Docker
Install docker and docker-compose.
From root folder:
```
docker-compose up -d
```

### Not docker
You can also run it directly from each folder if you know what you are doing.
TODO

### Client server for simulation view
To get the client server to work you need to change the address in `index.js` to point to the your server.

## Gateway registers
- Input register 1 holds the value of the tank. Divide by 32.768 to get the value in mm.
- Input register 2 holds the value of the inflow. Divide by 1638.4 to get the value in l/s
- Holding register 0 controls the value of the outflow. Multiply flow in l/s with 1638.4 to get the uint16 value to set.

## TODO
- [ ] Explain how to run it directly
- [ ] Simulation server code
  - [ ] Refactor
  - [ ] Add proper error handling
  - [ ] Improve comments
- [ ] Gateway code
  - [ ] Clean up and refactor gateway
- [ ] Web-interface code
  - [ ] Refactor and cleanup
  - [ ] Improve web-interface
- [ ] Write a short example of setup in GNS3
  - [ ] Explain OpenPLC setup
  - [ ] Explain Node-Red (SCADA-HMI) setup
