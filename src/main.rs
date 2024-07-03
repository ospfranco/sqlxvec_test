mod db;

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        // .filter_module("notary", log::LevelFilter::Debug)
        // .filter(Some("hyper_util"), log::LevelFilter::Off)
        // .filter(Some("reqwest"), log::LevelFilter::Off)
        .filter_level(log::LevelFilter::Debug)
        .init();

    let res = db::sqlite::run_test().await;
    println!("Result: {:?}", res);
}
