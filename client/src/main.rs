// 클라이언트를 바이너리로 실행
// 클라이언트 개발 테스트 용도
use kudrive_client::{event_loop, init, shutdown};

#[tokio::main]
async fn main() {
    init().await;

    if let Err(e) = event_loop().await {
        eprintln!("Event loop error: {:?}", e);

        println!("Client shutdown");
        shutdown().await;
    }
}
