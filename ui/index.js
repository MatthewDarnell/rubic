

const TIMEOUT_MS = 2000;

const updateServerRespondingStatus = connected => {
    const status = document.getElementById("connectedStatusSpan");
    status.innerHTML = connected === true ? '\u2705' : '\u274c';
}

const makeHttpRequest = (url, data = null, returnObjectWithStatus = null) => {
    return new Promise( (res, rej) => {
        try {
            const xhr = new XMLHttpRequest();
            xhr.onreadystatechange = () => {
                if (xhr.readyState === 4 && xhr.status === 200) {
                    updateServerRespondingStatus(true);
                    if (returnObjectWithStatus) return res({status: true, result: xhr.responseText})
                    else return res(xhr.responseText)
                } else if (xhr.status === 429) {
                    if (returnObjectWithStatus) {
                        updateServerRespondingStatus(true);
                        return rej({status: false, result: 'Too Many Requests!'})
                    }
                    else {
                        updateServerRespondingStatus(false);
                        return rej('Timed Out!')
                    }
                } else if (xhr.status >= 400) {
                    if (returnObjectWithStatus) return rej({status: false, result: xhr.status})
                    else return rej(status)
                }
            }

            xhr.onerror = function(e){
                updateServerRespondingStatus(false);
                return rej({status: false, result: "Unknown Error Occured. Server response not received."})
            };

            xhr.ontimeout = (e) => {
                updateServerRespondingStatus(false);
                if (returnObjectWithStatus) return rej({status: false, result: 'Timed Out!'})
                else return rej('Timed Out!')
            }
            xhr.open('GET',`${url}`, true);

            xhr.timeout = TIMEOUT_MS;
            xhr.send();
        } catch (error) {
            console.error(`Error making http request to ${url}${data ? '/' + data : ''} : <${error}>`)
            return rej(error)
        }
    })
}

const getNumConnectedPeers = async () => {
    try {
        const serverIp = document.getElementById("serverIp").value;
        const span = document.getElementById("numPeersSpan");
        const result = await makeHttpRequest(`${serverIp}/info`);
        span.innerHTML = `<b>${result}</b>`;
    } catch(error) {
    }
}

const timeConverter = timestamp => {
    const a = new Date(timestamp * 1000);
    const months = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'];
    const year = a.getFullYear();
    const month = months[a.getMonth()];
    const date = a.getDate();
    const hour = a.getHours();
    const min = a.getMinutes();
    const sec = a.getSeconds();
    const time = month + ' ' + date + ', ' + year + ' ' + hour + ':' + min + ':' + sec ;
    return time;
}

const getConnectedPeers = async () => {
    try {
        const serverIp = document.getElementById("serverIp").value;
        const table = document.getElementById("peerTable");
        const result = await makeHttpRequest(`${serverIp}/peers`);
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
            td3.innerText = `${timeResponded} : (${timeConverter(Number(timeResponded))})`;
            tr.appendChild(td);
            tr.appendChild(td2);
            tr.appendChild(td3);
            table.appendChild(tr);
        }
    } catch(error) {
    }
}


const getIdentities = async () => {
    const serverIp = document.getElementById("serverIp").value;
    const table = document.getElementById("identityTable");
    table.innerHTML = ""
    try {
        const result = await makeHttpRequest(`${serverIp}/identities`);
        table.innerHTML = "";
        const res = JSON.parse(result);
        for (const identity of res) {
            const tr = document.createElement("tr");
            const td = document.createElement("td");
            td.innerText = identity;
            tr.appendChild(td);

            const balanceTd = document.createElement("td");
            balanceTd.id = `${identity}:balance:td`

            tr.appendChild(balanceTd);
            table.appendChild(tr);
        }
        return res;
    } catch(error) {
    }
}

const getBalance = async identity => {
    const serverIp = document.getElementById("serverIp").value;
    const balanceTd = document.getElementById(`${identity}:balance:td`);
    balanceTd.innerHTML = ""
    try {
        const result = await makeHttpRequest(`${serverIp}/balance/${identity}`);
        const res = JSON.parse(result);
        if (res.every(v => v === res[0])) {
            balanceTd.innerHTML = `<b>${res[0]}</b>`
        } else {
            let html = `<span><b>Peer Responded Balance Mismatch: </b></span> [|`
            for(const r of res) {
                html += ` <b>${r}</b> |`
            }
            html += "]"
            balanceTd.innerHTML = html;
        }
        return res;
    } catch(error) {
    }
}

window.switchToElement = el => {
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
    const serverIp = document.getElementById("serverIp").value;
    document.getElementById("addNewPeerBtn").disabled = true;
    document.getElementById("newIdentityPreview").style.display = "none";
    const seed = document.getElementById("seedInput").value;
    const password = document.getElementById("passwordInput").value;
    makeHttpRequest(`${serverIp}/identity/add/${seed}`).then(result => {
        if(result === 200 || result === '200') {
            document.getElementById("newIdentityPreview").style.display = "none";
            document.getElementById("newIdentityPreviewSpan").innerText = ``;
            alert("Imported!");
        } else {
            alert(result);
        }
        document.getElementById("addNewPeerBtn").disabled = false;
    }).catch(result => {
        document.getElementById("addNewPeerBtn").disabled = false;
        alert(result);
    });
}

window.addNewPeer = () => {
    try {
        const serverIp = document.getElementById("serverIp").value;
        const ip = document.getElementById("addPeerIpInput").value;
        const portEl = document.getElementById("addPeerPortInput").value;
        console.log(ip)
        console.log(portEl)
        const port = portEl.length > 0 ? parseInt(portEl) : 21841;
        const values = ip.split('.')
        if(values.length > 0 && values.length !== 4) {
            alert("Invalid Ipv4 Address!")
            return;
        }
        if(port <= 0 || port > 99999) {
            alert("Invalid Port!")
            return;
        }
        const formattedPeerAddress = `${ip}:${port}`
        console.log(`${serverIp}/peers/add/${formattedPeerAddress}`)
        makeHttpRequest(`${serverIp}/peers/add/${formattedPeerAddress}`).then(result => {
            alert(result);
        }).catch(alert);
    } catch {}
}

const getLatestTick = async identity => {
    const serverIp = document.getElementById("serverIp").value;
    const latestTickSpan = document.getElementById("latestTickSpan");
    try {
        const result = await makeHttpRequest(`${serverIp}/tick`);
        console.log(result)
        latestTickSpan.innerHTML = `<b>${result}</b>`
        return result;
    } catch(error) {
    }
}

let numFuncsToCall = 4;
const intervalLoopFunction = () => {
    getLatestTick()
        .then(getNumConnectedPeers)
        .then(getConnectedPeers)
        .then(getIdentities)
        .then(async identities => {
            numFuncsToCall = 4 + identities.length;
            for(const id of identities) {
                await getBalance(id);
            }
        })
        .then(_ => {
            //Finished Update Loop
            setTimeout(intervalLoopFunction, (TIMEOUT_MS * numFuncsToCall) + 1000);
        })
        .catch(() => {
            setTimeout(intervalLoopFunction, (TIMEOUT_MS * numFuncsToCall) + 1000);
        })
}

window.onload = () => {
    console.log("Rubic JS Loaded!");
    setTimeout(intervalLoopFunction, (TIMEOUT_MS * numFuncsToCall) + 1000);
}