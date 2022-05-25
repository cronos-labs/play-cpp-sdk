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
const signature_secret = process.env.PAY_WEBHOOK_SIGNATURE_SECRET;
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
  if (pay_signature != undefined) {
    console.log(request.headers);
    let pay_signature_array = pay_signature.split(',');
    let timestamp = pay_signature_array[0].split('=')[1];
    let signature = pay_signature_array[1].split('=')[1];
    if (request.method == 'POST') {
      var body = '';
      request.on('data', function (data) {
        body += data;
      });
      request.on('end', function () {
        console.log(body);
        const hash = crypto.createHmac('sha256', signature_secret)
          .update(`${timestamp}.${body}`)
          .digest('hex');

        if (hash === signature) {
          let current_timestamp = Math.floor(new Date().getTime() / 1000);
          let timestamp_difference = current_timestamp - timestamp;
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
        // process.exit();
      });
    };
  } else {
    eventEmitter.emit('error', 'Invalid webhook request: no pay-signature in header');
    // process.exit();
  }
}


// handle the websocket client connection (e.g. from c++ client)
wss.on('connection', function wss_connection_listener(ws, req) {
  console.log(`Client ws://${req.socket.remoteAddress}:${req.socket.remotePort} connected`);

  eventEmitter.on('event', function status(data) {
    ws.send(`${data}`);
  });

  eventEmitter.on('error', function status(msg) {
    console.error(msg);
    ws.send(`error: ${msg}`);
  });

  ws.on('message', function message(data) {
    console.log('received: %s', data);
  });

});

server.listen(webhook_port, host);
console.log(`Listening at http://${host}:${webhook_port}`);
