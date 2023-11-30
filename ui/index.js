

const TIMEOUT_MS = 5000;
const TICK_OFFSET = 0;
const MAX_AMOUNT = 1000000000000000;
let globalLatestTick = 0;


const doArrayElementsAgree = (array, thresholdPercentage) => {
    const length = array.length;
    if(length < 3) {
        return -1;
    }
    const threshold = thresholdPercentage / 100;
    const numRequiredForQuorum = Math.ceil(threshold * length);
    const stateObj = {};
    for (const el of array) {
        if(!stateObj.hasOwnProperty(el)) {
            stateObj[el] = 1;
        } else {
            stateObj[el]++;
        }
    }
    let keys = Object.keys(stateObj)
    let balance = keys[0]
    let max = stateObj[keys[0]]
    for(let i = 1; i < keys.length; i++) {
        if(stateObj[keys[i]] > max) {
            balance = keys[i]
            max = stateObj[keys[i]]
        }
    }
    if(max > numRequiredForQuorum) {
        return parseInt(balance)
    } else {
        return -1;
    }
}


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
    try {
        const result = await makeHttpRequest(`${serverIp}/identities`);
        const res = JSON.parse(result);
        const newTableElements = [];
        for (let i = 0; i < res.length; i += 2) {
            let identity = res[i];
            let encrypted = res[i + 1];
            const tr = document.createElement("tr");
            const td = document.createElement("td");
            td.innerText = identity;
            tr.appendChild(td);

            const balanceTd = document.createElement("td");
            balanceTd.id = `${identity}:balance:td`

            let existingBalanceTd = document.getElementById(`${identity}:balance:td`);
            if(existingBalanceTd) {
                let existingBalance = existingBalanceTd.innerHTML ? existingBalanceTd.innerHTML : 0;
                balanceTd.innerHTML = existingBalance;
            } else {    //New Identity
                balanceTd.innerHTML = `<b>0</b>`;
            }

            tr.appendChild(balanceTd);
            const encryptedId = document.createElement("td");
            encryptedId.id = `${identity}:encrypted:td`
            encryptedId.innerHTML = encrypted;
            tr.appendChild(encryptedId);
            newTableElements.push(tr);
            //table.appendChild(tr);
        }
        table.innerHTML = "";
        for(const tr of newTableElements) {
            table.appendChild(tr);
        }
        return res;
    } catch(error) {
    }
}

const send = async (identity, isEncrypted) => {
    const passInput = document.getElementById("sendModalPasswordInput");
    if(passInput) {
        passInput.innerHTML = "";
    }
    if(isEncrypted) {
        const label = document.createElement("label");
        label.for = "sendModalPassword";
        label.innerHTML = "Master Password: ";

        const input = document.createElement("input");
        input.id = "sendModalPassword"
        input.type = "password";
        input.style.width = "80%";
        const td = document.createElement("td");
        td.appendChild(label);
        td.appendChild(input);
        passInput.appendChild(td);
    }


    document.getElementById("sendModalIdentitySpan").innerText = identity;
    const modal = document.getElementById('sendModal');
    modal.style.display = "block";
    var span = document.getElementsByClassName("close")[0];
    span.onclick = function() {
        modal.style.display = "none";
    }


}
const getBalance = async identity => {
    const serverIp = document.getElementById("serverIp").value;
    const balanceTd = document.getElementById(`${identity}:balance:td`);
    try {
        const result = await makeHttpRequest(`${serverIp}/balance/${identity}`);
        const res = JSON.parse(result);
        console.log(res)
        if(res.length < 3) {
            return balanceTd.innerHTML = `<span>Not Yet Reported</span>`
        }

        let reportedByTitle = "Reported By: <";
        for (let i = 0; i < res.length; i += 3) {
            console.log(`i=${i} pushing peer ${res[i+1]}`)
            reportedByTitle += (` ${res[i+1]}`);
        }
        reportedByTitle += (`> At Tick ${res[0]}`)


        const balanceArray = [];
        for (let i = 0; i < res.length; i += 3) {
            balanceArray.push(res[i+2]);
        }
        const isQuorumMet = doArrayElementsAgree(balanceArray, 66); // 2/3 of peers agree at this tick?

        const isEncrypted = document.getElementById(`${identity}:encrypted:td`).innerText.toLowerCase() === "true";
        if (balanceArray.every(v => v === res[0]) || isQuorumMet >= 0) {
            try {
                if(parseInt(balanceArray[0]) > 0) {
                    balanceTd.innerHTML = `<span title="${reportedByTitle}"><a href="#" onclick="send('${identity}', ${isEncrypted})"><b>${balanceArray[0]}</b> <span >\u27A4</span></a></span>`
                } else {
                    balanceTd.innerHTML = `<span title="${reportedByTitle}"><b>${balanceArray[0]}</b></span>`
                }
            } catch(err) {
                balanceTd.innerHTML = `<b>${balanceArray[0]}</b>`
            }
        } else {
            let html = `<span title="${reportedByTitle}"><b>Peer Responded Balance Mismatch: </b></span> [|`
            for(const r of balanceArray) {
                html += ` <b>${r}</b> |`
            }
            html += "]"
            balanceTd.innerHTML = html;
        }
        return res;
    } catch(error) {
        console.log(error)
    }
}

window.switchToElement = el => {
    const elToShow = document.getElementById(el);
    const identityDiv = document.getElementById('identityDiv');
    const peersDiv = document.getElementById('peersDiv');
    const settingsDiv = document.getElementById('settingsDiv');
    identityDiv.style.display = "none";
    peersDiv.style.display = "none";
    settingsDiv.style.display = "none";
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

window.generateRandomIdentity = () => {
    const serverIp = document.getElementById("serverIp").value;
    document.getElementById("generateRandomIdentityBtn").disabled = true;
    let password = document.getElementById("passwordInput").value;
    const isPasswordInputDisabled = document.getElementById("passwordInput").disabled;
    console.log(isPasswordInputDisabled)
    if(isPasswordInputDisabled || password.length < 4) {
        password = "0"
    }
    makeHttpRequest(`${serverIp}/identity/new/${password}`).then(result => {
        if(result === 200 || result === '200') {
            alert("Created!");
        } else {
            alert(result);
        }
        document.getElementById("generateRandomIdentityBtn").disabled = false;
    }).catch(result => {
        document.getElementById("generateRandomIdentityBtn").disabled = false;
        alert(result);
    });
}

window.addNewIdentity = () => {
    const serverIp = document.getElementById("serverIp").value;
    document.getElementById("addNewPeerBtn").disabled = true;
    document.getElementById("newIdentityPreview").style.display = "none";
    const seed = document.getElementById("seedInput").value;
    const password = document.getElementById("passwordInput").value || "";
    makeHttpRequest(`${serverIp}/identity/add/${seed}/${password}`).then(result => {
        if(result === 200 || result === '200') {
            document.getElementById("newIdentityPreview").style.display = "none";
            document.getElementById("newIdentityPreviewSpan").innerText = ``;
            document.getElementById("seedInput").value = ``;
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
        latestTickSpan.innerHTML = `<b>${result}</b>`
        globalLatestTick = parseInt(result);
        document.getElementById("sendModalExpirationTick").value = parseInt(result) + TICK_OFFSET;
        return result;
    } catch(error) {
    }
}

const getIsWalletEncrypted = async () => {
    const serverIp = document.getElementById("serverIp").value;
    const isWalletEncryptedSpan = document.getElementById("isWalletEncryptedSpan");
    try {
        const result = await makeHttpRequest(`${serverIp}/wallet/is_encrypted`);
        isWalletEncryptedSpan.innerHTML = result === 'true' ? '&#x1f512' : '&#x1f513'
        if(result === 'true') {
            //disable set master password btn
            document.getElementById('setDbPassBtn').innerText = "Password Already Set!";
            document.getElementById('setDbPassBtn').disabled = true;
            document.getElementById('setMasterPasswordInput').disabled = true;
            document.getElementById('passwordInput').disabled = false;
            if (document.getElementById('passwordInput').value == "You Can Set A Master Password In Settings" ) {
              document.getElementById('passwordInput').value = "";
            }
            document.getElementById('encryptAllIdentitiesInput').disabled = false;
            document.getElementById('encryptAllIdentitiesBtn').disabled = false;
        } else {
            document.getElementById('passwordInput').disabled = true;
            document.getElementById('passwordInput').value = "You Can Set A Master Password In Settings";
            document.getElementById('encryptAllIdentitiesInput').disabled = true;
            document.getElementById('encryptAllIdentitiesBtn').disabled = true;
            document.getElementById('setMasterPasswordInput').disabled = false;
            document.getElementById('setDbPassBtn').innerText = "Set Password";
            document.getElementById('setDbPassBtn').disabled = false;

        }
        return result;
    } catch(error) {
        isWalletEncryptedSpan.innerHTML = '\u2753';
    }
}

window.setMasterPassword = () => {
    try {
        const serverIp = document.getElementById("serverIp").value;
        const password = document.getElementById("setMasterPasswordInput").value;
        document.getElementById("setMasterPasswordInput").value = "";
        document.getElementById('setMasterPasswordInput').disabled = true;
        document.getElementById('setDbPassBtn').innerText = "Setting Password...";
        document.getElementById('setDbPassBtn').disabled = true;
        makeHttpRequest(`${serverIp}/wallet/set_master_password/${password}`)
            .then(result => {
                alert(result);
            })
    } catch(error) {
        alert(error);
    }
}

window.encryptAllIdentities = () => {
    try {
        const serverIp = document.getElementById("serverIp").value;
        const password = document.getElementById("encryptAllIdentitiesInput").value;
        makeHttpRequest(`${serverIp}/wallet/encrypt/${password}`)
            .then(result => {
                alert(result);
                document.getElementById("encryptAllIdentitiesInput").value = "";
            })
    } catch(error) {
        alert(error);
    }
}

window.exportDb = () => {
    try {
        const serverIp = document.getElementById("serverIp").value;
        const password = document.getElementById("exportSettingsPasswordInput").value;
        const decrypt = password.length > 0 ? true : false;
        if(decrypt) {
            makeHttpRequest(`${serverIp}/wallet/download/${password}`)
                .then(result => {
                    if(result.split(",").length < 2) {
                        alert("Invalid Password!")
                    } else {
                        let csvContent = "data:text/csv;charset=utf-8," + result;
                        var encodedUri = encodeURI(csvContent);
                        const downloadLink = document.createElement("a");
                        downloadLink.href = encodedUri;
                        downloadLink.download = "rubic-db-decrypted.csv";
                        document.body.appendChild(downloadLink);
                        downloadLink.click();
                        document.body.removeChild(downloadLink);

                    }
                })
        } else {
            makeHttpRequest(`${serverIp}/wallet/download/0`)
                .then(result => {
                    let csvContent = "data:text/csv;charset=utf-8," + result;
                    var encodedUri = encodeURI(csvContent);
                    const downloadLink = document.createElement("a");
                    downloadLink.href = encodedUri;
                    downloadLink.download = "rubic-db-encrypted.csv";
                    document.body.appendChild(downloadLink);
                    downloadLink.click();
                    document.body.removeChild(downloadLink);

                })
        }
    } catch(error) {
        alert(error);
    }
}


window.initiateTransfer = async () => {
    try {
        const serverIp = document.getElementById("serverIp").value;
        const expirationTick = parseInt(document.getElementById("sendModalExpirationTick").value);
        const sourceIdentity = document.getElementById("sendModalIdentitySpan").innerText;
        const destinationIdentity = document.getElementById("sendModalDestinationInput").value;
        const password = document.getElementById("sendModalPassword").value || "0";
        const amountToSend = parseInt(document.getElementById("sendModalAmountInput").value);
        if(isNaN(expirationTick) || expirationTick <= 0 || expirationTick < (globalLatestTick + TICK_OFFSET)) {
            alert("Invalid Expiration Tick!");
            return;
        }
        if(isNaN(amountToSend) || amountToSend <= 0 || amountToSend > MAX_AMOUNT) {
            alert("Invalid Amount To Send!");
            return;
        }
        if(sourceIdentity.length !== 60) {
            alert("Invalid Source Identity!");
            return;
        }
        if(destinationIdentity.length !== 60) {
            alert("Invalid Destination Identity!");
            return;
        }
        document.getElementById("sendQubicsBtn").disabled = true;
        const result = await makeHttpRequest(`${serverIp}/transfer/${sourceIdentity}/${destinationIdentity}/${amountToSend}/${expirationTick}/${password}`);
        document.getElementById("sendQubicsBtn").disabled = false;
        alert(result);
    } catch(error) {
        console.log(`Error in initiateTransfer!`);
        console.log(error);
    }
}

/*
    Runtime
*/
let numFuncsToCall = 4;

const statusInfoLoopFunction = () => {
    getLatestTick()
        .then(getIsWalletEncrypted)
        .then(getNumConnectedPeers)
        .then(getConnectedPeers)
        .then(_ => {
            //Finished Update Loop
            setTimeout(statusInfoLoopFunction, 7000);
        })
        .catch(() => {
            setTimeout(statusInfoLoopFunction, 3000);
        })
}


let loopCounter = 11;
const intervalLoopFunction = () => {
    loopCounter++;
    getIdentities()
        .then(async identities => {
            if(loopCounter > 10) {
                identities = identities.filter(x => x.length > 10);
                numFuncsToCall = 1 + identities.length;
                for(const id of identities) {
                    await getBalance(id);
                }
                loopCounter = 0;
            } else {
                numFuncsToCall = 1;
            }
        })
        .then(_ => {
            //Finished Update Loop
            setTimeout(intervalLoopFunction, 5000);
        })
        .catch(() => {
            setTimeout(intervalLoopFunction, 1000);
        })
}

//Clicking Outside of Send Modal Should Close It.
window.onclick = function(event) {
    const modal = document.getElementById('sendModal');
    if (event.target == modal) {
        modal.style.display = "none";
    }
}

window.onload = () => {
    console.log("Rubic JS Loaded!");
    statusInfoLoopFunction();
    intervalLoopFunction();
}