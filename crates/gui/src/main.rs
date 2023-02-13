use dioxus::prelude::{
    dioxus_elements::{button, img, input},
    *,
};
use log::{info, LevelFilter};
use private_relay::server::start_server;
use private_relay::*;
use std::sync::mpsc::{Receiver as MpscReceiver, Sender as MpscSender};
use std::thread;
use std::{
    borrow::Borrow,
    fs::{self, OpenOptions},
};
use std::{
    io::Write,
    sync::{Arc, Condvar, Mutex},
};
use std::{sync::mpsc as syncmpsc, thread::JoinHandle};
use toml;

fn main() {
    dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");

    // simple_logger::init_with_level(log::Level::Debug).unwrap();
    dioxus::desktop::launch_cfg(App, |c| {
        c.with_window(|w| {
            w.with_title("FlyCastle")
                .with_resizable(true)
                .with_inner_size(dioxus::desktop::wry::application::dpi::LogicalSize::new(
                    400.0, 300.0,
                ))
        })
    });
}

static App: Component<()> = |cx| {
    let (public_key, set_public_key) = use_state(&cx, || "".to_owned());
    let (is_running, set_is_running) = use_state(&cx, || false);

    let default_config_file_path = "../relay/config.toml".to_string();
    let mut config_file_path = "./config.toml".to_string();
    if let Ok(metadata) = fs::metadata(config_file_path.clone()) {
        println!("Config file exists with size {} bytes.", metadata.len());
    } else {
        println!("Config file does not exist. create a new one");
        config_file_path = default_config_file_path;
    }
    let config_file_arg: Option<String> = Some(config_file_path.clone());
    let settings = config::Settings::new(&config_file_arg);

    let mut is_pubkey_exits = false;
    let mut first_pubkey = "".to_string();
    let mut short_public_key = "";
    let pubkey_whitelist = settings.authorization.pubkey_whitelist;
    if let Some(vec) = pubkey_whitelist {
        if let Some(first_item) = vec.first() {
            is_pubkey_exits = true;
            // use the first item
            first_pubkey = first_item.to_string();
            short_public_key = &first_pubkey[0..25];
        }
    }

    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = pair.clone();

    let start_relay = move |pair: Arc<(Mutex<bool>, Condvar)>,
                            pair2: Arc<(Mutex<bool>, Condvar)>| {
        info!("Public key: {}", public_key);

        let default_config_file_path = "../relay/config.toml".to_string();
        let mut config_file_path = "./config.toml".to_string();
        let save_config_file_path = "./config.toml".to_string();
        if let Ok(metadata) = fs::metadata(config_file_path.clone()) {
            println!("Config file exists with size {} bytes.", metadata.len());
        } else {
            println!("Config file does not exist. create a new one");
            config_file_path = default_config_file_path;
        }

        let config_file_arg: Option<String> = Some(config_file_path.clone());
        let mut settings = config::Settings::new(&config_file_arg);

        match settings.authorization.pubkey_whitelist {
            None => {
                settings.authorization.pubkey_whitelist = Some(vec![public_key.to_owned()]);
                let config_str = toml::to_string(&settings).expect("Failed to serialize config");
                if fs::metadata(save_config_file_path.clone()).is_err() {
                    let mut config_file = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(save_config_file_path)
                        .expect("Failed to create config file");

                    config_file
                        .write_all(config_str.as_bytes())
                        .expect("Failed to write config file");

                    println!("save new config file!");
                } else {
                    let mut config_file = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open(save_config_file_path)
                        .expect("Failed to open config file");

                    config_file
                        .write_all(config_str.as_bytes())
                        .expect("Failed to write config file");

                    println!("save config");
                }
            }
            _ => {}
        }

        // we should have a 'control plane' channel to monitor and bump
        // the server.  this will let us do stuff like clear the database,
        // shutdown, etc.; for now all this does is initiate shutdown if
        // `()` is sent.  This will change in the future, this is just a
        // stopgap to shutdown the relay when it is used as a library.
        let (_, ctrl_rx): (MpscSender<()>, MpscReceiver<()>) = syncmpsc::channel();
        // run this in a new thread
        let handle = thread::spawn(move || {
            let &(ref lock, ref cvar) = &*pair2;
            let mut started = lock.lock().unwrap();
            while !*started {
                started = cvar.wait(started).unwrap();
            }

            let _svr = start_server(&settings, ctrl_rx);
        });

        let &(ref lock, ref cvar) = &*pair;
        let mut started = lock.lock().unwrap();
        *started = true;
        cvar.notify_one();
        // block on nostr thread to finish.
        // handle.join().unwrap();
        handle
    };

    let stop_relay = move |pair: Arc<(Mutex<bool>, Condvar)>| {
        // Stop the thread if needed
        let &(ref lock, ref cvar) = &*pair;
        let mut started = lock.lock().unwrap();
        *started = false;
        cvar.notify_one();

        // Wait for the child thread to finish
        //handle.join().unwrap();
    };

    let btn_class = if *is_running { "running" } else { "stop" };
    rsx!(cx, div {
        style { [include_str!("./style.css")] }
        header {
            img {
                src: "https://nostr.build/i/nostr.build_4c12e3b96ceaf9c2d709df6d9e6e8f170c7d0dd6271665f2afe0191d129f396c.jpeg",
                alt: "Header Image",
            }
        }
        main {
            rsx!(
                div {
                    h3 {"Your own little relay"}
                    if !is_pubkey_exits {
                        rsx!(
                            input {
                                placeholder: "Enter your public key",
                                value: "{public_key}",
                                oninput: move |e| set_public_key(e.value.clone()),
                             }
                        )
                    }else{
                        rsx!(p{
                            "{short_public_key}.."
                        })
                    }

                    button {
                        class: "{btn_class}",
                        onclick: move |_| {
                            if *is_running {
                                //set_is_running(false);
                                //stop_relay(pair.clone());
                            }else{
                                set_is_running(true);
                                start_relay(pair.clone(), pair2.clone());
                            }
                        },
                       rsx!([if *is_running { "Running" } else { "Start" }] )
                    }
                }
            )
        }

    })
};
