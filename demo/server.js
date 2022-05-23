const crypto = require('crypto');
const http = require('http');
// The local test port exposed by ngrok
const port = process.env.PAY_NGROK_PORT;
const host = '127.0.0.1';
// You can subscribe to the events by going to your Merchant Dashboard and add a new webhook
// subscription. Retrieve your endpoint’s Signature Secret from your Merchant Dashboard's
// Webhooks settings
const signature_secret = process.env.PAY_WEBHOOK_SIGNATURE_SECRET;
// tolerance between the current timestamp and the received timestamp in seconds
const timestamp_tolerance = 2;

const server = http.createServer(function (request, response) {
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
          } else {
            console.error(`Expired webhook request: ${timestamp_difference} > ${timestamp_tolerance}`);
          }
        } else {
          console.error("Invalid signture");
        }
        process.exit();
      });
    };
  } else {
    console.error("Invalid webhook request: no pay-signature in header");
    process.exit();
  }
});

server.listen(port, host);
console.log(`Listening at http://${host}:${port}`);
