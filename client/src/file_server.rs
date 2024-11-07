pub struct FileServer;

impl FileServer {
    pub fn new() -> Self {
        Self
    }

    pub async fn start(&self) {
        println!("File server started.");
    }

    pub async fn stop(&self) {
        println!("File server stopped.");
    }
}
