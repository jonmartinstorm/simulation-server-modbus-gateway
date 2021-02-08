# simulation-server-modbus-gateway
A simulation server for a water tank with a modbus gateway for control

## How to run
Install docker and docker-compose.
From root folder:
```
docker-compose up -d
```

You can also run it directly from each folder if you know what you are doing.

## Gateway registers
Input register 1 holds the value of the tank. Divide by 32.768 to get the value in mm.
Input register 2 holds the value of the inflow. Divide by 1638.4 to get the value in l/s
Holding register 0 controls the value of the outflow. Multiply flow in l/s with 1638.4 to get the uint16 value to set.

## TODO
- [ ] Explain how to run it directly
- [ ] Refactor
- [ ] Improve comments
- [ ] Clean up and refactor gateway
- [ ] Improve web-interface
