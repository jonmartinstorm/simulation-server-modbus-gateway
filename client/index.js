function WebSocketTest(ws, tank) {
    // Let us open a web socket

    let tank_h = tank.getElementById("tank").getAttribute("height");
    let tank_y = tank.getElementById("tank").getAttribute("y");

    ws.onopen = function() {
        // Web Socket is connected, send data using send()
        ws.send("Hei til serveren");
        console.log("Message is sent...");
    };

    ws.onmessage = function(evt) {
        var received_msg = evt.data;
        //document.getElementById("message").textContent = received_msg;
        tank_values = JSON.parse(received_msg);
        tank.getElementById("inflow-text").textContent = "Inflow: " + tank_values["inflow"].toFixed(1) + "l/s";
        tank.getElementById("height-text").textContent = "Height: " + tank_values["height"].toFixed(0) + "mm";
        tank.getElementById("setpoint-text").textContent = "Setpoint: " + tank_values["set_level"].toFixed(0) + "mm";
        tank.getElementById("level-text").textContent = "Level: " + tank_values["level"].toFixed(0) + "mm";
        tank.getElementById("outflow-text").textContent = "Outflow: " + tank_values["outflow"].toFixed(1) + "l/s";

        let [w_y, w_h] = CalculateLevel(tank_h, tank_y, tank_values["level"], tank_values["height"]);
        tank.getElementById("water").setAttribute("height", w_h);
        tank.getElementById("water").setAttribute("y", w_y);
        //console.log(received_msg);
    };

    ws.onclose = function() {

        // websocket is closed.
        console.log("Connection is closed...");
    };
}

function CalculateLevel(tank_h, tank_y, level, real_max) {
    let tank_real_h = tank_h / real_max;
    let h = tank_real_h * level;
    let y = tank_y - (h - tank_h);
    return [y, h];
}

var ws = new WebSocket("ws://localhost:7799");

var tank = document.getElementById("tankImage");
tank.addEventListener('load', () => {
    console.log("SVG loaded!");
    //console.log(tank.getSVGDocument().getElementById("water"));
    tanksvg = tank.getSVGDocument();
    tanksvg.getElementById("inflow-text").style["font-size"] = "3px";
    tanksvg.getElementById("height-text").style["font-size"] = "3px";
    tanksvg.getElementById("level-text").style["font-size"] = "3px";
    tanksvg.getElementById("setpoint-text").style["font-size"] = "3px";
    tanksvg.getElementById("outflow-text").style["font-size"] = "3px";

   

    WebSocketTest(ws, tanksvg);
});

