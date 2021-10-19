use arrows::boxes::BoxStore;
#[async_std::main]
async fn main() {
    let boxstore = self::run().await;
    let _process_dir = boxstore.get_dir().await;
    //loop {
    // }
}

async fn run() -> BoxStore {
    BoxStore::init().await
}
