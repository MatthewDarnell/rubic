

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


const getIdentities = async () => {
    const serverIp = document.getElementById("serverIp").value;
    console.log('server Ip: ' + serverIp)
    const table = document.getElementById("identityTable");
    table.innerHTML = ""
    try {
        const result = await makeHttpRequest(`${serverIp}/identities`);
        console.log(result);
        table.innerHTML = "";
        const res = JSON.parse(result);
        for (const identity of res) {
            console.log('adding ' + identity)
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
        console.log(`Error in getIdentities: ${error}`)
    }
}

const getBalance = async identity => {
    const serverIp = document.getElementById("serverIp").value;
    console.log('server Ip: ' + serverIp)
    const balanceTd = document.getElementById(`${identity}:balance:td`);
    balanceTd.innerHTML = ""
    try {
        console.log('fetching balance')
        const result = await makeHttpRequest(`${serverIp}/balance/${identity}`);
        const res = JSON.parse(result);
        console.log(res)
        if (res.every(v => v === res[0])) {
            balanceTd.innerHTML = `<b>${res[0]}</b>`
        } else {
            let html = `<span><b>Peer Responded Balance Mismatch: </b></span> [|`
            for(const r of res) {
                html += ` <b>${r}</b> |`
            }
            html += "]"
            console.log(html)
            balanceTd.innerHTML = html;
        }
        return res;
    } catch(error) {
        console.log(`Error in getBalance: ${error}`)
    }
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
    const serverIp = document.getElementById("serverIp").value;
    document.getElementById("newIdentityPreview").style.display = "none";
    const seed = document.getElementById("seedInput").value;
    const password = document.getElementById("passwordInput").value;
    console.log(`${serverIp}/identity/add/${seed}`)
    makeHttpRequest(`${serverIp}/identity/add/${seed}`).then(result => {
        if(result === 200 || result === '200') {
            document.getElementById("newIdentityPreview").style.display = "none";
            document.getElementById("newIdentityPreviewSpan").innerText = ``;
            alert("Imported!");
        } else {
            alert(result);
        }
    }).catch(alert);
}

window.onload = () => {
    console.log('loaded')
    setInterval(() => {
        getNumConnectedPeers();
        getConnectedPeers();
        getIdentities()
            .then(async identities => {
                for(const id of identities) {
                    await getBalance(id);
                }
            })
            .catch(_ => {});
    }, 3000);

}