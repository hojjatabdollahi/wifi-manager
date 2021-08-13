use online;
use rocket::tokio::sync::Mutex;
use rocket::tokio::time::{sleep, Duration};
use rocket::State;
use rocket::{get, post, routes};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Arc;
// use wifi_rs::{prelude::*, WiFi};
use wifiscanner;
use color_eyre::{Report, eyre::eyre};
use tracing::info;
use tracing_subscriber::EnvFilter;

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

// type Result<T> = std::result::Result<T, WifiError>;


// #[derive(Debug, Clone)]
// struct WifiError;

// impl fmt::Display for WifiError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "Wifi error")
//     }
// }

// impl std::error::Error for WifiError {
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         None
//     }
// }


type WiFiState<'r> = &'r State<SWifi>;

#[get("/delay/<seconds>")]
async fn delay<'r>(state: WiFiState<'r>, seconds: u64) -> String {
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
async fn dev<'r>(state: WiFiState<'r>) -> String {
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
async fn isonline<'r>(state: WiFiState<'r>) -> String {
    match state.data1.try_lock() {
        Ok(mut lock) => match online::check(Some(2)).await {
            Ok(()) => {
                *lock += 1;
                format!("online")
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
async fn wifion<'r>(state: WiFiState<'r>) -> String {
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
async fn wifioff<'r>(state: WiFiState<'r>) -> String {
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
async fn iswifienabled<'r>(state: WiFiState<'r>) -> String {
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
    state: WiFiState<'r>,
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
async fn disconnect<'r>(state: WiFiState<'r>) -> Json<Response> {
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
async fn ssids<'r>(state: WiFiState<'r>) -> Json<Response> {
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

#[get("/ssid")]
async fn current_ssid<'r>(state: WiFiState<'r>) -> Json<Response> {
    let output;
    match state.data1.try_lock() {
        Ok(mut lock) => {
            match get_current_ssid() {
                Ok(ssid_bool) => {
                    output = Response {
                        message: format!("Done"),
                        data: vec![(ssid_bool, true)],
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
async fn main() -> Result<(), Report> {
    setup()?;
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
                iswifienabled,
                current_ssid
            ],
        )
        .manage(s_wifi)
        .launch()
        .await;

        Ok(())
}

fn setup() -> Result<(), Report> {
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }
    color_eyre::install()?;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    Ok(())
}

fn disconnect_wifi() -> Result<String, Report> {
    let device = wifiscanner::get_dev()?;
    // let config = Some(Config {
    //     interface: Some(&device),
    // });
    // let wifi = WiFi::new(config);
    let output = Command::new("nmcli")
        .args(&["d", "disconnect", "ifname", &device])
        .output()
        .map_err(|_err| eyre!("Couldn't disconnect"))?;
    if String::from_utf8_lossy(&output.stdout)
        .as_ref()
        .contains("disconnect") {
        Ok("Disconnection Successful".to_string())
    } else {
        Err(eyre!("Couldn't disconnect"))
    }
}

fn get_ssids() -> Result<Vec<(String, bool)>, Report> {
    let mut results: Vec<(String, bool)> = vec![];
    let wifis =  wifiscanner::scan()?;
    for wifi in wifis {
        println!("{:?}", wifi);
        results.push((wifi.ssid.to_string(), wifi.security.is_empty()));
    }
    Ok(results)
}


fn get_current_ssid() -> Result<String, Report> {

    let output = Command::new("nmcli")
        .args(&["-t", "-f", "active,ssid", "dev", "wifi"])
        .output()?;

    info!(?output);

    Ok(String::from_utf8_lossy(&output.stdout)
        .replace(" ", "")
        .replace("\n", "")
        .contains("enabled").to_string())
    // let mut results: Vec<(String, bool)> = vec![];
    // match wifiscanner::scan() {
    //     Ok(wifis) => {
    //         for wifi in &wifis {
    //             println!("{:?}", wifi);
    //             results.push((wifi.ssid.to_string(), wifi.security.is_empty()));
    //         }
    //         Ok(results)
    //     }
    //     Err(_e) => Err(WifiError),
    // }
}


fn connect_wifi(ssid: String, passwd: String) -> Result<String, Report> {
    let device = wifiscanner::get_dev()?; 
    if !is_wifi_enabled()? {
        return Err(eyre!("Wifi is disabled!"));
    }
    // let config = Some(Config {
    //     interface: Some(&device),
    // });
    // let mut wifi = WiFi::new(config);

    let output = Command::new("nmcli")
        .args(&[
            "d",
            "wifi",
            "connect",
            &ssid,
            "password",
            &passwd,
            "ifname",
            &device,
        ])
        .output()
        .map_err(|_err| eyre!("couldn't connect"))?;

    if !String::from_utf8_lossy(&output.stdout)
        .as_ref()
        .contains("successfully activated")
    {
        return Ok("did not connect".to_string());
    }
    Ok("Connected".to_string())
}


fn is_wifi_enabled() -> Result<bool, Report> {
    let output = Command::new("nmcli")
        .args(&["radio", "wifi"])
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout)
        .replace(" ", "")
        .replace("\n", "")
        .contains("enabled"))
}

/// Turn on the wireless network adapter.
fn turn_on() -> Result<(), Report> {
    Command::new("nmcli")
        .args(&["radio", "wifi", "on"])
        .output()?;

    Ok(())
}

/// Turn off the wireless network adapter.
fn turn_off() -> Result<(), Report> {
    Command::new("nmcli")
        .args(&["radio", "wifi", "off"])
        .output()?;

    Ok(())
}
