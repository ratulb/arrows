use arrows::boxes::BoxStore;
use arrows::Address;
use async_std::fs::File;
use async_std::io::ReadExt;
use async_std::sync::Mutex;
#[async_std::main]
async fn main() {
    let boxstore = self::run().await;
    let process_dir = boxstore.get_dir().await;
    println!(
        "BoxStore: {:#?}, {:?}",
        boxstore,
        process_dir.exists().await
    );
    let addr = Address::new("actor1");
    println!("Address: {:#?}", addr);
    //loop {
    println!("In actors");
    let mut buffer = [0; 1024];
    let file = File::open("foo.txt").await;
    type_of(&file);
    let locked_file = Mutex::new(file.unwrap());
    let mut awaited = locked_file.lock().await;
    let n = awaited.read(&mut buffer).await;
    let n = n.unwrap();
    println!("The bytes: {:?}", &buffer[..n]);
    // }
}

async fn run() -> BoxStore {
    BoxStore::init().await
}
fn type_of<T>(_: &T) {
    println!("The type is {}", std::any::type_name::<T>());
}

fn write(file_name: &str) -> std::io::Result<usize> {
    Ok(0)
}
