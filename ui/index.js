

const makeHttpRequest = (url, data = null, returnObjectWithStatus = null) => {
    console.log('got url ' + url)
    return new Promise( (res, rej) => {
        try {
            const xhr = new XMLHttpRequest();
            xhr.onreadystatechange = () => {
                if (xhr.readyState === 4 && xhr.status === 200) {
                    if (returnObjectWithStatus) return res({status: true, result: xhr.responseText})
                    else return res(xhr.responseText)
                } else if (xhr.status === 429) {
                    if (returnObjectWithStatus) return rej({status: false, result: 'Too Many Requests!'})
                    else return rej('Timed Out!')
                } else if (xhr.status >= 400) {
                    if (returnObjectWithStatus) return rej({status: false, result: xhr.status})
                    else return rej(status)
                }
            }
            xhr.ontimeout = (e) => {
                if (returnObjectWithStatus) return rej({status: false, result: 'Timed Out!'})
                else return rej('Timed Out!')
            }

            console.log(`Fetching Data From: <${url}>`)

            xhr.open('GET',`${url}`, true);

            xhr.timeout = 5000;
            xhr.send();
        } catch (error) {
            console.error(`Error making http request to ${url}${data ? '/' + data : ''} : <${error}>`)
            return rej(error)
        }
    })
}

const getNumConnectedPeers = () => {
    const serverIp = document.getElementById("serverIp").value;

    console.log('server Ip: ' + serverIp)
    const span = document.getElementById("numPeersSpan");
    makeHttpRequest(`${serverIp}/info`).then(result => {
        span.innerText = result;
    }).catch(alert);
}


const getConnectedPeers = () => {
    const serverIp = document.getElementById("serverIp").value;

    console.log('server Ip: ' + serverIp)
    const table = document.getElementById("peerTable");
    makeHttpRequest(`${serverIp}/peers`).then(result => {
        table.innerHTML = "";
        const res = JSON.parse(result);
        for (const peer of res) {
            const ip = peer[1];
            const nick = peer[2].length > 1 ? peer[2] : "<NickName Not Set>";
            const timeResponded = peer[5];
            const tr = document.createElement("tr");
            const td = document.createElement("td");
            const td2 = document.createElement("td");
            const td3 = document.createElement("td");
            td.innerText = ip;
            td2.innerText = nick;
            td3.innerText = timeResponded;
            tr.appendChild(td);
            tr.appendChild(td2);
            tr.appendChild(td3);
            table.appendChild(tr);
        }
    }).catch(alert);
}


const getIdentities = () => {
    const serverIp = document.getElementById("serverIp").value;
    console.log('server Ip: ' + serverIp)
    const table = document.getElementById("identityTable");
    makeHttpRequest(`${serverIp}/identities`).then(result => {
        console.log(result)
        /*
        table.innerHTML = "";
        const res = JSON.parse(result);
        for (const peer of res) {
            const ip = peer[1];
            const nick = peer[2].length > 1 ? peer[2] : "<NickName Not Set>";
            const timeResponded = peer[5];
            const tr = document.createElement("tr");
            const td = document.createElement("td");
            const td2 = document.createElement("td");
            const td3 = document.createElement("td");
            td.innerText = ip;
            td2.innerText = nick;
            td3.innerText = timeResponded;
            tr.appendChild(td);
            tr.appendChild(td2);
            tr.appendChild(td3);
            table.appendChild(tr);
        }
         */
    }).catch(alert);
}

window.switchToElement = el => {
    console.log("Showing " + el)
    const elToShow = document.getElementById(el);
    const identityDiv = document.getElementById('identityDiv');
    const peersDiv = document.getElementById('peersDiv');
    identityDiv.style.display = "none";
    peersDiv.style.display = "none";
    elToShow.style.display = "block";
}

window.previewNewIdentity = () => {
    document.getElementById("importNewIdentityBtn").disabled = true;
    const serverIp = document.getElementById("serverIp").value;
    document.getElementById("newIdentityPreview").style.display = "none";
    const seed = document.getElementById("seedInput").value;
    const password = document.getElementById("passwordInput").value;
    console.log(`${serverIp}/identity/from_seed/${seed}`)

    if(seed.length !== 55) {
        document.getElementById("newIdentityPreview").style.display = "block";
        document.getElementById("newIdentityPreviewSpan").innerText = "Invalid Seed!";
    } else {
        makeHttpRequest(`${serverIp}/identity/from_seed/${seed}`).then(result => {
            if (result === "AARQXIKNFIEZZEMOAVNVSUINZXAAXYBZZXVSWYOYIETZVPVKJPARMKTEKLKJ") { //invalid seed
                document.getElementById("newIdentityPreview").style.display = "block";
                document.getElementById("newIdentityPreviewSpan").innerText = "Invalid Seed!";
            } else {
                document.getElementById("newIdentityPreview").style.display = "block";
                document.getElementById("newIdentityPreviewSpan").innerText = `Importing Identity <${result}>`;
                document.getElementById("importNewIdentityBtn").disabled = false;
            }
        }).catch(alert);
    }
}

window.addNewIdentity = () => {
    console.log('importing!')
    /*
    const serverIp = document.getElementById("serverIp").value;
    document.getElementById("newIdentityPreview").style.display = "none";
    const seed = document.getElementById("seedInput").value;
    const password = document.getElementById("passwordInput").value;
    console.log(`${serverIp}/identity/from_seed/${seed}`)
    makeHttpRequest(`${serverIp}/identity/from_seed/${seed}`).then(result => {
        if(result === "AARQXIKNFIEZZEMOAVNVSUINZXAAXYBZZXVSWYOYIETZVPVKJPARMKTEKLKJ") { //invalid seed
            document.getElementById("newIdentityPreview").style.display = "block";
            document.getElementById("newIdentityPreviewSpan").innerText = "Invalid Seed!";
        } else {
            document.getElementById("newIdentityPreview").style.display = "block";
            document.getElementById("newIdentityPreviewSpan").innerText = `Importing Identity <${result}>`;
        }
    }).catch(alert);

     */

}
window.onload = () => {
    console.log('loaded')
    setInterval(() => {
        getNumConnectedPeers();
        getConnectedPeers();
        getIdentities();
    }, 10000);

}