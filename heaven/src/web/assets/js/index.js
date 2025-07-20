async function abortJob(job) {
    await fetch("/port/" + job + "/abort");
}

function createReloader(divId, page) {
    async function reloadHeader() {
        const response = await fetch(page);
        const data = await response.text();
        document.getElementById(divId).innerHTML = data;
    }

    setInterval(reloadHeader, 1000);
}

createReloader("portstatus", "portstatus.html");