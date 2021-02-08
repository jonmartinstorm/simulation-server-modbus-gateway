function WebSocketTest(ws) {
    // Let us open a web socket

    ws.onopen = function() {
        // Web Socket is connected, send data using send()
        ws.send("Hei til serveren");
        console.log("Message is sent...");
    };

    ws.onmessage = function(evt) {
        var received_msg = evt.data;
        document.getElementById("message").textContent = received_msg;
        console.log(received_msg);
    };

    ws.onclose = function() {

        // websocket is closed.
        console.log("Connection is closed...");
    };
}

var ws = new WebSocket("ws://localhost:7799");

WebSocketTest(ws);

function sendFunny(ws) {
    ws.send("Funny");
}