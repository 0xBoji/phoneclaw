use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use log::LevelFilter;
use std::path::PathBuf;
use std::sync::Once;
use std::thread;

static INIT: Once = Once::new();

#[no_mangle]
pub extern "system" fn Java_com_pocketclaw_app_RustBridge_startServer(
    mut env: JNIEnv,
    _class: JClass,
    config_path: JString,
) -> jstring {
    // Initialize Android logger once
    INIT.call_once(|| {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(LevelFilter::Info)
                .with_tag("PocketClaw"),
        );
    });

    // Convert Java string to Rust PathBuf
    let config_path_str: String = env
        .get_string(&config_path)
        .expect("Couldn't get java string!")
        .into();
    let config_path = PathBuf::from(config_path_str);

    log::info!("Starting PocketClaw Server with config: {:?}", config_path);

    // Spawn the server in a new thread because start_server blocks
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            if let Err(e) = pocketclaw_cli::start_server(Some(config_path)).await {
                log::error!("Server failed: {}", e);
            }
        });
    });

    let output = env
        .new_string("Server started")
        .expect("Couldn't create java string!");
    output.into_raw()
}

#[no_mangle]
pub extern "system" fn Java_com_pocketclaw_app_RustBridge_stopServer(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    // TODO: Implement graceful shutdown mechanism in pocketclaw-cli first
    log::info!("Stop server requested (not fully implemented)");
    
    let output = env
        .new_string("Stop signal sent")
        .expect("Couldn't create java string!");
    output.into_raw()
}
