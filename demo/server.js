const crypto = require('crypto');
const ws = require('ws');
const http = require('http');
const EventEmitter = require("events");

// The local test port exposed by ngrok
const webhook_port = process.env.PAY_WEBHOOK_PORT;
const host = '127.0.0.1';
// You can subscribe to the events by going to your Merchant Dashboard and add a new webhook
// subscription. Retrieve your endpoint’s Signature Secret from your Merchant Dashboard's
// Webhooks settings
// Webhook `SIGNATURE SECRET` of one of the `PAYLOAD URL`s
const SIGNATURE_SECRET = process.env.PAY_WEBHOOK_SIGNATURE_SECRET;
// tolerance between the current timestamp and the received timestamp in seconds
const timestamp_tolerance = 2;

// create a http server
const server = http.createServer(http_request_listener);
// create a ws server by sharing a http server
const wss = new ws.WebSocketServer({ server });
// create an event emitter
const eventEmitter = new EventEmitter();

function http_request_listener(request, response) {
  let pay_signature = request.headers['pay-signature'];
  if (pay_signature != undefined && pay_signature.split(',').length > 1) {
    console.log(request.headers);
    let pay_signature_array = pay_signature.split(',');

    let timestamp = '';
    let signatures = [];

    // The Pay-Signature header contains a timestamp and one or more signatures. The timestamp
    // is prefixed by t=, and each signature is prefixed by a scheme. Schemes start with v1.
    //
    // To prevent downgrade attacks, you should ignore all schemes that are not v1.
    for (ele of pay_signature_array) {
      let ele_array = ele.split('=');
      if (ele_array.length == 2) {
        if (ele_array[0] == 't') {
          timestamp = ele_array[1];
        } else if (ele_array[0] == 'v1') {
          signatures.push(ele_array[1]);
        }
      }
    }

    if (timestamp == '' || signatures.length == 0) {
      eventEmitter.emit('error', 'Invalid pay signture');
    } else if (request.method == 'POST') {
      var body = '';
      request.on('data', function (data) {
        body += data;
      });
      request.on('end', function () {
        console.log(body);
        // Compute an HMAC with the SHA256 hash function. Use the endpoint’s Signature Secret
        // as the key, and use the aforesaid concatenated string as the message.
        const hash = crypto.createHmac('sha256', SIGNATURE_SECRET)
          .update(`${timestamp}.${body}`)
          .digest();

        // TODO In this example, we only compare one signature: suppose only one signature in
        // `pay-signature`.
        const sig = Buffer.from(signatures[0], 'hex');

        // To protect against timing attacks, use a constant-time string comparison to compare
        // the expected signature to each of the received signatures.
        if (crypto.timingSafeEqual(hash, sig)) {
          let current_timestamp = Math.floor(new Date().getTime() / 1000);
          let timestamp_difference = current_timestamp - timestamp;
          // Additionally, you may compute the difference between the current timestamp and the
          // received timestamp, then decide if the difference is within your tolerance.
          if (timestamp_difference <= timestamp_tolerance) {
            response.writeHead(200, { 'Content-Type': 'text/html' });
            response.end('post received');
            console.log("Valid webhook request");
            eventEmitter.emit('event', body);
          } else {
            eventEmitter.emit('error', `Expired webhook request: ${timestamp_difference} > ${timestamp_tolerance}`);
          }
        } else {
          eventEmitter.emit('error', 'Invalid Signture');
        }
      });
    };

  } else {
    eventEmitter.emit('error', 'Invalid webhook request: no pay-signature in header');
  }
}

let websocket = null;
// handle the websocket client connection (e.g. from c++ client)
wss.on('connection', function wss_connection_listener(ws, req) {
  console.log(`Client ws://${req.socket.remoteAddress}:${req.socket.remotePort} connected`);
  // get ws after detecting connenction event
  websocket = ws;
  websocket.on('message', function message(data) {
    console.log('received: %s', data);
  });

});

eventEmitter.on('event', function status(data) {
  if (websocket) {
    websocket.send(`${data}`);
  }
});

eventEmitter.on('error', function status(msg) {
  console.error(msg);
  if (websocket) {
    websocket.send(`error: ${msg}`);
  }
});

server.listen(webhook_port, host);
console.log(`Listening at http://${host}:${webhook_port}`);
