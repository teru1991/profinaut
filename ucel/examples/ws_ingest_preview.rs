use ucel_core::IngestStreamKey;
use ucel_subscription_planner::build_desired_plan;

fn main() {
    let streams = vec![IngestStreamKey {
        exchange: "binance".into(),
        family: "spot".into(),
        channel: "trades".into(),
        symbol: "BTCUSDT".into(),
        shard: 0,
        auth_scope: "public".into(),
    }];
    let planned = build_desired_plan(streams);
    println!("planned_streams={}", planned.len());
}
