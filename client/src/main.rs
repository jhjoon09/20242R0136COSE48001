// 클라이언트를 바이너리로 실행
// 클라이언트 개발 테스트 용도
use kudrive_client::Client;

#[tokio::main]
async fn main() {
    let client = Client::new();

    println!("Starting client...");
    client.start().await;

    println!("Client shutdown");
    client.shutdown().await;
}
