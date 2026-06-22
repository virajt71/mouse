#![allow(dead_code)]
#![allow(clippy::manual_range_contains, clippy::invisible_characters)]
pub mod battery;
pub mod bluetooth;
pub mod cache;
pub mod config;
pub mod detection;
pub mod engine;
pub mod flow;
pub mod hidpp;
pub mod input;
pub mod lock_ext;
pub mod receiver;
pub mod worker;

pub use self::engine::Engine;

pub static TOKIO_RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> = std::sync::LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime")
});
