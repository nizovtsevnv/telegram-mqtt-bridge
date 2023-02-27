// https://core.telegram.org/bots/api#available-methods

use log::{debug, error, info, warn};
use rumqttc::v5::mqttbytes::v5::Packet;
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::Event::Incoming;
use rumqttc::v5::{AsyncClient, MqttOptions};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;

const TELEGRAM_API_DOMAIN: &str = "https://api.telegram.org";

#[derive(Serialize, Deserialize, Debug)]
struct MqttMessage {
    method: String,
    payload: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TgMessage {
    chat_id: u32,
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TgPoll {
    offset: u64,
    timeout: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let client_id = env::var("CLIENT_ID").unwrap_or("telegram-mqtt-bridge".to_string());

    let queue_host = env::var("QUEUE_HOST").unwrap_or("localhost".to_string());

    let queue_polling_timeout = env::var("QUEUE_POLLING_TIMEOUT").unwrap_or("60".to_string()).parse::<u64>().unwrap_or_else(|_| {
        error!("Need to set a valid QUEUE_POLLING_TIMEOUT environment variable (default: 60)");
        exit(1);
    });

    let queue_port = env::var("QUEUE_PORT").unwrap_or("1883".to_string()).parse::<u16>().unwrap_or_else(|_| {
        error!("Need to set a valid QUEUE_PORT environment variable (default: 1883)");
        exit(1);
    });

    let send_to_queue = env::var("SEND_TO_QUEUE").unwrap_or("messages-from-telegram".to_string());

    let send_to_telegram = env::var("SEND_TO_TELEGRAM").unwrap_or("messages-to-telegram".to_string());

    let telegram_polling_timeout = env::var("TELEGRAM_POLLING_TIMEOUT").unwrap_or("60".to_string()).parse::<u8>().unwrap_or_else(|_| {
        error!("Need to set a valid TELEGRAM_POLLING_TIMEOUT environment variable (default: 60)");
        exit(1);
    });

    let telegram_token = env::var("TELEGRAM_TOKEN").unwrap_or_else(|_| {
        error!("Need to set TELEGRAM_TOKEN environment variable");
        exit(1);
    });

    info!("Telegram <-> MQTT Gateway");

    // Telegram client initialization
    let http_handle = Arc::new(reqwest::Client::new());

    // MQTT client initialization
    let mut mqtt_options =
        MqttOptions::new(client_id, queue_host.to_string(), queue_port);
    mqtt_options.set_keep_alive(Duration::from_secs(queue_polling_timeout));
    let (mqtt_client, mut eventloop) = AsyncClient::new(mqtt_options, 1);
    let mqtt_handle = Arc::new(mqtt_client);

    // MQTT listening cycle forwarding messages to Telegram
    let http = http_handle.clone();
    let mqtt = mqtt_handle.clone();
    let queue_name = send_to_telegram.clone();
    let token = telegram_token.clone();
    tokio::spawn(async move {
        loop {
            info!("Subscribe to '{}' MQTT topic", &queue_name);

            match mqtt.subscribe(&queue_name, QoS::AtMostOnce).await {
                Ok(_) => {
                    while let Ok(event) = eventloop.poll().await {
                        debug!(
                            "Received event from '{}' MQTT topic: {:?}",
                            &queue_name, event
                        );

                        if let Incoming(boxed_packet) = event {
                            if let Packet::Publish(packet_publish, _) = boxed_packet.as_ref() {
                                debug!(
                                    "Received message from '{}' MQTT topic: {:?}",
                                    &queue_name, packet_publish
                                );

                                match std::str::from_utf8(packet_publish.payload.as_ref()) {
                                    Ok(string) => {
                                        let parts: Vec<&str> = string.splitn(2, "\n").collect();
                                        if parts.len() == 2 {
                                            let method = parts[0].to_owned();
                                            let body = parts[1].to_owned();

                                            debug!("Send to Telegram API '{}': {:?}", method, body);

                                            match http
                                                .post(format!(
                                                    "{}/bot{}/{}",
                                                    TELEGRAM_API_DOMAIN, &token, method
                                                ))
                                                .header("Content-Type", "application/json")
                                                .body(body)
                                                .send()
                                                .await
                                            {
                                                Ok(response) => {
                                                    debug!(
                                                        "Result from Telegram API '{}': {:?}",
                                                        method, response
                                                    );
                                                }
                                                Err(error) => {
                                                    warn!(
                                                        "Can't send to Telegram API '{}': {:?}",
                                                        method, error
                                                    );
                                                }
                                            }
                                        } else {
                                            warn!("Queue message must contain Telegram API method and JSON payload splitted by new line character");
                                        }
                                    }
                                    Err(_) => {
                                        warn!("Can't get UTF-8 string from raw payload");
                                    }
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    warn!("Can't subscribe to '{}' MQTT topic", send_to_telegram);
                }
            }
        }
    });

    // Telegram listening cycle forwarding messages to MQTT
    let http = http_handle.clone();
    let mqtt = mqtt_handle.clone();
    let queue_name = send_to_queue.clone();
    let token = telegram_token.clone();
    let mut telegram_update_id: u64 = 1;

    loop {
        info!("Request updates from Telegram API");
        match http
            .post(format!(
                "{}/bot{}/{}",
                TELEGRAM_API_DOMAIN, &token, "getUpdates"
            ))
            .json(&TgPoll {
                offset: telegram_update_id,
                timeout: telegram_polling_timeout,
            })
            .send()
            .await
        {
            Ok(response) => {
                match response.text().await {
                    Ok(serialized) => {
                        match serde_json::from_str::<serde_json::Value>(serialized.as_str()) {
                            Ok(deserialized) => {
                                if let Some(updates) = deserialized["result"].as_array() {
                                    for update in updates {
                                        debug!("Process update from Telegram API: {}", update);

                                        match update["update_id"].as_u64() {
                                            Some(update_id) => {
                                                match serde_json::to_string(update) {
                                                    Ok(serialized) => {
                                                        match mqtt
                                                            .publish(
                                                                &queue_name,
                                                                QoS::AtLeastOnce,
                                                                false,
                                                                serialized,
                                                            )
                                                            .await
                                                        {
                                                            Ok(result) => {
                                                                debug!("Telegram API update sending to MQTT result: {:?}", result);
                                                            }
                                                            Err(error) => {
                                                                warn!("Can't send Telegram API update to MQTT: {}", error);
                                                            }
                                                        }
                                                    }
                                                    Err(error) => {
                                                        warn!("Can't serialize Telegram API update: {}", error);
                                                    }
                                                }

                                                // Обновление минимального требуемого идентификатора события для следующего запроса
                                                telegram_update_id = update_id + 1;
                                            }
                                            None => {
                                                warn!("No identifier in Telegram API update");
                                            }
                                        }
                                    }
                                } else {
                                    debug!("No new updates on Telegram API: {}", deserialized);
                                }
                            }
                            Err(error) => {
                                warn!("Can't parse JSON updates from Telegram API: {:?}", error);
                            }
                        }
                    }
                    Err(error) => {
                        warn!("Can't get updates payload from Telegram API: {:?}", error);
                    }
                }
            }
            Err(error) => {
                warn!("Can't get updates from Telegram API: {:?}", error);
            }
        }
    }
}
