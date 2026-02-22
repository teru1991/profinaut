use symbol_master::config::AppConfig;
use symbol_master::supervisor::Supervisor;

#[tokio::main]
async fn main() {
    let _config = AppConfig { exchanges: vec![] };
    let _supervisor = Supervisor::default();
}
