var term = new Terminal();
term.open(document.getElementById('terminal'));

function connect() {
    console.log("Connecting to socket...");
    const webSocket = new WebSocket("serial");
    webSocket.onclose = function (e) {
        console.log('Socket is closed. Reconnect will be attempted in 1 second.', e.reason);
        setTimeout(function () {
            connect();
        }, 1000);
    }
    webSocket.onopen = function () {
        console.log("Socket connected!");
        const attachAddon = new AttachAddon.AttachAddon(webSocket);
        term.loadAddon(attachAddon);
    }
    webSocket.onerror = function(err) {
        console.error('Socket encountered error: ', err.message, 'Closing socket');
        ws.close();
    };
}

connect();

async function abortJob() {
    const response = await fetch("abort");
    if (response.ok) {
        term.clear();
    }
}

function createReloader(divId, page) {
    async function reloadHeader() {
        const response = await fetch(page);
        const data = await response.text();
        document.getElementById(divId).innerHTML = data;
    }

    setInterval(reloadHeader, 1000);
}

createReloader("header", "header.html");
createReloader("devinfo", "devinfo.html");
