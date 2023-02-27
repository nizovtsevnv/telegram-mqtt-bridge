<h1 align="center">
  <br>
  <img src="https://github.com/nizovtsevnv/telegram-mqtt-bridge/raw/main/assets/tg-mqtt-bridge.png" alt="Telegram MQTT bridge" width="200"></a>
  <br>
  telegram-mqtt-bridge
  <br>
</h1>

<h4 align="center">Forwarding of messages between Telegram Bot API and MQTT broker.</h4>

<p align="center">
  <a href="https://saythanks.io/to/nizovtsevnv">
      <img src="https://img.shields.io/badge/SayThanks.io-%E2%98%BC-1EAEDB.svg">
  </a>
  <a href="https://buymeacoffee.com/nizovtsevnv">
    <img src="https://img.shields.io/badge/$-donate-ff69b4.svg?maxAge=2592000&amp;style=flat">
  </a>
</p>

<p align="center">
  <a href="#key-features">Key Features</a> •
  <a href="#how-to-use">How To Use</a> •
  <a href="#support">Support</a> •
  <a href="#license">License</a>
</p>

## Key Features

* Connecting to Telegram Bot API by token authentication
* Connecting to MQTT broker without authentication
* Forwarding any messages from Telegram Bot API to MQTT in the same JSON format as Telegram Update object
* Requesting any Telegram Bot API methods with JSON data from the MQTT message
* Configuring all gateway options via environment variables

## How To Use

To clone and run this application, you'll need [Git](https://git-scm.com) and [Rust](https://www.rust-lang.org/tools/install) installed on your computer.

From your command line:

```bash
# Clone this repository
$ git clone https://github.com/nizovtsevnv/telegram-mqtt-bridge

# Go into the repository
$ cd telegram-mqtt-bridge

# Install dependencies and run the app
$ RUST_LOG=info TELEGRAM_TOKEN=CHANGE_IT_TO_YOUR_VALUE QUEUE_HOST=localhost QUEUE_PORT=1883 SEND_TO_TELEGRAM=messages-to-telegram SEND_TO_QUEUE=messages-from-telegram cargo run
```

To set up options use environment variables:
* (optional) **QUEUE_HOST** - MQTT broker, default value - "**localhost**"
* (optional) **QUEUE_POLLING_TIMEOUT** - polling request timeout, default value - **60** seconds
* (optional) **QUEUE_PORT** - MQTT broker port, default value - **1883**
* (optional) **RUST_LOG** - logging level, default value - "**error**"
* (optional) **SEND_TO_QUEUE** - queue name for messages from Telegram to MQTT, default value - "**messages-from-telegram**"
* (optional) **SEND_TO_TELEGRAM** - queue name for messages from MQTT to Telegram, default value - "**messages-to-telegram**"
* (optional) **TELEGRAM_POLLING_TIMEOUT** - polling request timeout, default value - **60** seconds
* (required) **TELEGRAM_TOKEN** - authentication token for [Telegram Bot API](https://t.me/botfather)

## Support

<a href="https://www.buymeacoffee.com/nizovtsevnv" target="_blank"><img src="https://www.buymeacoffee.com/assets/img/custom_images/purple_img.png" alt="Buy Me A Coffee" style="height: 41px !important;width: 174px !important;box-shadow: 0px 3px 2px 0px rgba(190, 190, 190, 0.5) !important;-webkit-box-shadow: 0px 3px 2px 0px rgba(190, 190, 190, 0.5) !important;" ></a>

## License

MIT

---
