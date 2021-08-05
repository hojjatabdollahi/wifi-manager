use online;
use rocket::tokio::sync::Mutex;
use rocket::tokio::time::{sleep, Duration};
use rocket::State;
use rocket::{get, post, routes};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::process::Command;
use std::sync::Arc;
use wifi_rs::{prelude::*, WiFi};
use wifiscanner;

struct SWifi {
    data1: Arc<Mutex<usize>>,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world"
}

#[derive(Serialize, Deserialize)]
struct Response {
    #[serde(rename = "Message")]
    message: String,
    #[serde(rename = "Data")]
    data: Vec<(String, bool)>,
}

type Result<T> = std::result::Result<T, WifiError>;

#[derive(Debug, Clone)]
struct WifiError;

impl fmt::Display for WifiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Wifi error")
    }
}

#[get("/delay/<seconds>")]
async fn delay<'r>(state: State<'r, SWifi>, seconds: u64) -> String {
    match state.data1.try_lock() {
        Ok(mut lock) => {
            sleep(Duration::from_secs(seconds)).await;
            *lock += 1;
            format!("Waited for {} seconds", seconds)
        }
        Err(_) => {
            format!("I'm busy")
        }
    }
}

#[get("/dev")]
async fn dev<'r>(state: State<'r, SWifi>) -> String {
    match state.data1.try_lock() {
        Ok(mut lock) => {
            let dev = wifiscanner::get_dev().expect("Couldn't get the device name");
            *lock += 1;
            format!("The device name: {}", dev)
        }
        Err(_) => {
            format!("I'm busy")
        }
    }
}

#[get("/isonline")]
async fn isonline<'r>(state: State<'r, SWifi>) -> String {
    match state.data1.try_lock() {
        Ok(mut lock) => match online::online(Some(2)).await {
            Ok(dev) => {
                *lock += 1;
                format!("The device name: {}", dev)
            }
            Err(_e) => {
                *lock += 1;
                format!("We are not connected to the internet")
            }
        },
        Err(_) => {
            format!("I'm busy")
        }
    }
}

#[get("/wifion")]
async fn wifion<'r>(state: State<'r, SWifi>) -> String {
    match state.data1.try_lock() {
        Ok(mut lock) => match turn_on() {
            Ok(_) => {
                *lock += 1;
                format!("Wifi is on")
            }
            Err(_e) => {
                format!("Error happened when turing the wifi on")
            }
        },
        Err(_e) => {
            format!("I'm busy")
        }
    }
}

#[get("/wifioff")]
async fn wifioff<'r>(state: State<'r, SWifi>) -> String {
    match state.data1.try_lock() {
        Ok(mut lock) => match turn_off() {
            Ok(_) => {
                *lock += 1;
                format!("Wifi is off")
            }
            Err(_e) => {
                format!("Error happened when turning the wifi off")
            }
        },
        Err(_e) => {
            format!("I'm busy")
        }
    }
}

#[get("/iswifienabled")]
async fn iswifienabled<'r>(state: State<'r, SWifi>) -> String {
    match state.data1.try_lock() {
        Ok(mut lock) => match is_wifi_enabled() {
            Ok(true) => {
                *lock += 1;
                format!("Wifi is enabled")
            }
            Ok(false) => {
                *lock += 1;
                format!("Wifi is not enabled")
            }
            Err(_e) => {
                format!("Error happened when checking wifi status")
            }
        },
        Err(_e) => {
            format!("I'm busy")
        }
    }
}

#[derive(Deserialize, Debug)]
struct ConnectionInfo {
    ssid: String,
    passwd: String,
}

#[post("/connect", format = "json", data = "<connection_info>")]
async fn connect<'r>(
    state: State<'r, SWifi>,
    connection_info: Json<ConnectionInfo>,
) -> Json<Response> {
    let output;
    match state.data1.try_lock() {
        Ok(mut lock) => {
            match connect_wifi(
                connection_info.ssid.to_string(),
                connection_info.passwd.to_string(),
            ) {
                Ok(msg) => {
                    output = format!("Done: {}", msg);
                }
                Err(_) => {
                    output = format!("Connection failed");
                }
            }
            *lock += 1;
            Json(Response {
                message: output,
                data: vec![],
            })
        }
        Err(_) => Json(Response {
            message: format!("I'm busy"),
            data: vec![],
        }),
    }
}

#[get("/disconnect")]
async fn disconnect<'r>(state: State<'r, SWifi>) -> Json<Response> {
    let output;
    match state.data1.try_lock() {
        Ok(mut lock) => {
            match disconnect_wifi() {
                Ok(msg) => {
                    output = format!("Done: {}", msg);
                }
                Err(_) => {
                    output = format!("Disconnect failed");
                }
            }
            *lock += 1;
            Json(Response {
                message: output,
                data: vec![],
            })
        }
        Err(_) => Json(Response {
            message: format!("I'm busy"),
            data: vec![],
        }),
    }
}

#[get("/ssids")]
async fn ssids<'r>(state: State<'r, SWifi>) -> Json<Response> {
    let output;
    match state.data1.try_lock() {
        Ok(mut lock) => {
            match get_ssids() {
                Ok(ssid_vec) => {
                    output = Response {
                        message: format!("Done"),
                        data: ssid_vec,
                    };
                }
                Err(_) => {
                    output = Response {
                        message: format!("Failed to get the SSIDs"),
                        data: vec![],
                    };
                }
            }
            *lock += 1;
            Json(output)
        }
        Err(_) => Json(Response {
            message: format!("I'm busy"),
            data: vec![],
        }),
    }
}

#[rocket::main]
async fn main() {
    let s_wifi = SWifi {
        data1: Arc::new(Mutex::new(0)),
    };
    let _ = rocket::build()
        .mount(
            "/",
            routes![
                index,
                delay,
                connect,
                disconnect,
                ssids,
                dev,
                isonline,
                wifion,
                wifioff,
                iswifienabled
            ],
        )
        .manage(s_wifi)
        .launch()
        .await;
}

fn disconnect_wifi() -> Result<String> {
    match wifiscanner::get_dev() {
        Ok(device) => {
            let config = Some(Config {
                interface: Some(&device),
            });
            let wifi = WiFi::new(config);
            match wifi.disconnect() {
                Ok(result) => {
                    if result == true {
                        Ok("Disconnection Successful".to_string())
                    } else {
                        Err(WifiError)
                    }
                }
                Err(_err) => Err(WifiError),
            }
        }
        Err(_err) => Err(WifiError),
    }
}

fn get_ssids() -> Result<Vec<(String, bool)>> {
    let mut results: Vec<(String, bool)> = vec![];
    match wifiscanner::scan() {
        Ok(wifis) => {
            for wifi in &wifis {
                println!("{:?}", wifi);
                results.push((wifi.ssid.to_string(), wifi.security.is_empty()));
            }
            Ok(results)
        }
        Err(_e) => Err(WifiError),
    }
}

fn connect_wifi(ssid: String, passwd: String) -> Result<String> {
    match wifiscanner::get_dev() {
        Ok(device) => {
            let config = Some(Config {
                interface: Some(&device),
            });
            let mut wifi = WiFi::new(config);
            match wifi.connect(&ssid, &passwd) {
                Ok(result) => {
                    if result == true {
                        Ok("Connection Successful".to_string())
                    } else {
                        Err(WifiError)
                    }
                }
                Err(_err) => Err(WifiError),
            }
        }
        Err(_err) => Err(WifiError),
    }
}

fn is_wifi_enabled() -> Result<bool> {
    let output = Command::new("nmcli")
        .args(&["radio", "wifi"])
        .output()
        .map_err(|err| WifiError)?;

    Ok(String::from_utf8_lossy(&output.stdout)
        .replace(" ", "")
        .replace("\n", "")
        .contains("enabled"))
}

/// Turn on the wireless network adapter.
fn turn_on() -> Result<()> {
    Command::new("nmcli")
        .args(&["radio", "wifi", "on"])
        .output()
        .map_err(|err| WifiError)?;

    Ok(())
}

/// Turn off the wireless network adapter.
fn turn_off() -> Result<()> {
    Command::new("nmcli")
        .args(&["radio", "wifi", "off"])
        .output()
        .map_err(|err| WifiError)?;

    Ok(())
}
