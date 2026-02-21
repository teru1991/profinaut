use ucel_registry::invoker::Invoker;

fn main() {
    let invoker = Invoker::default();
    for venue in invoker.list_venues().unwrap() {
        let ids = invoker.list_ids(&venue).unwrap();
        println!("{} {}", venue, ids.len());
    }
}
