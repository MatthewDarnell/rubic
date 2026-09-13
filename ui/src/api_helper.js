export const doArrayElementsAgree = (array, thresholdPercentage) => {
  const length = array.length;
  if (length < 2) {
    return -1;
  }
  const threshold = thresholdPercentage / 100;
  const numRequiredForQuorum = Math.ceil(threshold * length);
  const stateObj = {};
  for (const el of array) {
    if (!stateObj.hasOwnProperty(el)) {
      stateObj[el] = 1;
    } else {
      stateObj[el]++;
    }
  }
  let keys = Object.keys(stateObj);
  let balance = keys[0];
  let max = stateObj[keys[0]];
  for (let i = 1; i < keys.length; i++) {
    if (stateObj[keys[i]] > max) {
      balance = keys[i];
      max = stateObj[keys[i]];
    }
  }
  if (max > numRequiredForQuorum) {
    return parseInt(balance);
  } else {
    return -1;
  }
};

export const timeConverter = (timestamp) => {
  const a = new Date(timestamp * 1000);
  const months = [
    'Jan',
    'Feb',
    'Mar',
    'Apr',
    'May',
    'Jun',
    'Jul',
    'Aug',
    'Sep',
    'Oct',
    'Nov',
    'Dec',
  ];
  const year = a.getFullYear();
  const month = months[a.getMonth()];
  const date = a.getDate();
  const hour = a.getHours();
  const min = a.getMinutes();
  const sec = a.getSeconds();
  const time =
    month + ' ' + date + ', ' + year + ' ' + hour + ':' + min + ':' + sec;
  return time;
};
