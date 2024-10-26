#![feature(let_chains)]
#![warn(unused_results)]

pub mod app;
pub mod bt_manager;
pub mod config;
pub mod globals;
pub mod helpers;
pub mod keymaps;
pub mod models;
pub mod theme;
pub mod views;

#[tokio::main]
async fn main() {
    app::App::new().await.init().await.run().await.unwrap();
}
