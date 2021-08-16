use online;
use rocket::tokio::sync::Mutex;
use rocket::tokio::time::{sleep, Duration};
use rocket::State;
use rocket::{get, post, routes};
use rocket::serde::json::Json;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Arc;
use wifiscanner;
use color_eyre::{Report, eyre::eyre};
use tracing::info;
use tracing_subscriber::EnvFilter;
use substring::Substring;

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
            format!("busy")
        }
    }
}

#[get("/isonline")]
async fn isonline<'r>(state: WiFiState<'r>) -> Json<Response> {
    match state.data1.try_lock() {
        Ok(mut lock) => match online::check(Some(2)).await {
            Ok(()) => {
                *lock += 1;
                // format!("online")

                Json(Response {
                    message: "online".to_string(),
                    data: vec![("online".to_string(), true)],
                })
            }
            Err(_e) => {
                *lock += 1;
                // format!("We are not connected to the internet")
                Json(Response {
                    message: "offline".to_string(),
                    data: vec![("offline".to_string(), false)],
                })
            }
        },
        Err(_) => {
            Json(Response {
                message: "busy".to_string(),
                data: vec![("busy".to_string(), false)],
            })
        }
    }
}

#[get("/wifion")]
async fn wifion<'r>(state: WiFiState<'r>) -> Json<Response>  {
    match state.data1.try_lock() {
        Ok(mut lock) => match turn_on() {
            Ok(_) => {
                *lock += 1;
                Json(Response {
                    message: "on".to_string(),
                    data: vec![("on".to_string(), true)],
                })
            }
            Err(_e) => {
                Json(Response {
                    message: "error".to_string(),
                    data: vec![("error".to_string(), false)],
                })
            }
        },
        Err(_e) => {
            Json(Response {
                message: "busy".to_string(),
                data: vec![("busy".to_string(), false)],
            })
        }
    }
}

#[get("/wifioff")]
async fn wifioff<'r>(state: WiFiState<'r>) -> Json<Response> {
    match state.data1.try_lock() {
        Ok(mut lock) => match turn_off() {
            Ok(_) => {
                *lock += 1;
                Json(Response {
                    message: "off".to_string(),
                    data: vec![("off".to_string(), true)],
                })
            }
            Err(_e) => {
                Json(Response {
                    message: "error".to_string(),
                    data: vec![("error".to_string(), false)],
                })
            }
        },
        Err(_e) => {
            Json(Response {
                message: "busy".to_string(),
                data: vec![("busy".to_string(), false)],
            })
        }
    }
}

#[get("/iswifienabled")]
async fn iswifienabled<'r>(state: WiFiState<'r>) -> Json<Response> {
    match state.data1.try_lock() {
        Ok(mut lock) => match is_wifi_enabled() {
            Ok(true) => {
                *lock += 1;
                Json(Response {
                    message: "enabled".to_string(),
                    data: vec![("enabled".to_string(), true)],
                })
            }
            Ok(false) => {
                *lock += 1;
                Json(Response {
                    message: "disabled".to_string(),
                    data: vec![("disabled".to_string(), false)],
                })
            }
            Err(_e) => {
                Json(Response {
                    message: "error".to_string(),
                    data: vec![("error".to_string(), false)],
                })
            }
        },
        Err(_e) => {
            Json(Response {
                message: "busy".to_string(),
                data: vec![("busy".to_string(), false)],
            })
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
    match state.data1.try_lock() {
        Ok(mut lock) => {
            *lock += 1;
            match connect_wifi(
                connection_info.ssid.to_string(),
                connection_info.passwd.to_string(),
            ) {
                Ok(_msg) => {
                    return Json(Response {
                        message: "done".to_string(),
                        data: vec![("done".to_string(), true)],
                    });
                }
                Err(_) => {
                    return Json(Response {
                        message: "error".to_string(),
                        data: vec![("error".to_string(), false)],
                    });
                }
            }
        }
        Err(_) => Json(Response {
            message: format!("busy"),
            data: vec![("busy".to_string(), false)],
        }),
    }
}

#[get("/disconnect")]
async fn disconnect<'r>(state: WiFiState<'r>) -> Json<Response> {
    match state.data1.try_lock() {
        Ok(mut lock) => {
            *lock += 1;
            match disconnect_wifi() {
                Ok(_msg) => {
                    return Json(Response {
                        message: "done".to_string(),
                        data: vec![("done".to_string(), true)],
                    });
                }
                Err(_) => {
                    return Json(Response {
                        message: "error".to_string(),
                        data: vec![("error".to_string(), false)],
                    });
                }
            }
        }
        Err(_) => Json(Response {
            message: format!("busy"),
            data: vec![("busy".to_string(), false)],
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
                        message: format!("error"),
                        data: vec![("error".to_string(), false)],
                    };
                }
            }
            *lock += 1;
            Json(output)
        }
        Err(_) => Json(Response {
            message: format!("busy"),
            data: vec![("busy".to_string(), false)],
        }),
    }
}

#[get("/ssid")]
async fn current_ssid<'r>(state: WiFiState<'r>) -> Json<Response> {
    let output;
    match state.data1.try_lock() {
        Ok(mut lock) => {
            match get_current_ssid() {
                Ok(Some(ssid)) => {
                    output = Response {
                        message: format!("Done"),
                        data: vec![(ssid, true)],
                    };
                }
                Ok(None) => {
                    output = Response {
                        message: format!("Done"),
                        data: vec![(String::new(), false)],
                    };
                }
                Err(_) => {
                    output = Response {
                        message: format!("error"),
                        data: vec![("error".to_string(), false)],
                    };
                }
            }
            *lock += 1;
            Json(output)
        }
        Err(_) => Json(Response {
            message: format!("busy"),
            data: vec![("busy".to_string(), false)],
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


fn get_current_ssid() -> Result<Option<String>, Report> {

    let output = Command::new("nmcli")
        .args(&["-t", "-f", "active,ssid", "dev", "wifi"])
        .output()?;

    info!(?output);

    let output = String::from_utf8_lossy(&output.stdout);
    Ok(output 
        .split('\n')
        .into_iter()
        .filter(|&str| str.starts_with("yes:"))
        .map(|found| found.substring(4,found.len()).to_string())
        .nth(0))
}


fn connect_wifi(ssid: String, passwd: String) -> Result<String, Report> {
    let device = wifiscanner::get_dev()?; 
    if !is_wifi_enabled()? {
        return Err(eyre!("Wifi is disabled!"));
    }

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
