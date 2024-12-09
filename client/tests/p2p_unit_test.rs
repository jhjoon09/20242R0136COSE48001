use core::panic;
use kudrive_client::event::ClientEvent;
use kudrive_client::p2p::{P2PTransport, P2pCommand, P2pStatus};
use libp2p::PeerId;
use rand::{distributions::Alphanumeric, Rng};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::oneshot::channel;
use tokio::sync::OnceCell;
use tokio::time;

const MAX_DIAL_RETRY: u32 = 5;
const LOCAL_RELAY_ADDR: &str =
    "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooWA768LzHMatxkjD1f9DrYW375GZJr6MHPCNEdDtHeTNRt";

const SERVER_WARMUP_TIME: u64 = 5;
const WARMUP_TIME: u64 = 10;
const TEST_TIMEOUT: u64 = 10;
const TEST_WAIT_TIMEOUT: u64 = 10;
const TEST_RECIVER_TIME_DELAY: u64 = 5;

const DUMMY_CONTENT: &[u8; 30] = b"Test content for file transfer";
const DUMMY_FILE_PATH: &str = "./dummy_file.txt";
const DUMMY_RECV_FILE_PATH: &str = "./test_dummy/dummy_file.txt";
const INT_DUMMY_FILE_PATH: &str = "./dummy_file1.txt";
const INT_DUMMY_RECV_FILE_PATH: &str = "./test_dummy1/dummy_file1.txt";

static SERVER_INSTANCE: OnceCell<TestServer> = OnceCell::const_new();

struct TestServer {
    process: Child,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}

async fn start_test_server() -> TestServer {
    let process = Command::new("cargo")
        .arg("run")
        .arg("--manifest-path")
        .arg("../server/Cargo.toml")
        .arg("--")
        .arg("--test-p2p")
        .spawn()
        .expect("Failed to start server");

    // Wait for init
    time::sleep(Duration::from_secs(SERVER_WARMUP_TIME)).await;

    TestServer { process }
}

async fn wait_test_server() -> &'static TestServer {
    SERVER_INSTANCE
        .get_or_init(|| async { start_test_server().await })
        .await
}

#[tokio::test]
async fn test_get_local_peer_id() {
    let _server = wait_test_server().await;

    let client = setup_mock_client(&generate_rand_id()).await;

    let _ = client.warm_up_with_delay(WARMUP_TIME).await;

    let (tx, rx) = channel();

    let command = P2pCommand::GetId { response_tx: tx };
    client
        .command_tx
        .send(command)
        .await
        .expect("Failed to send command");

    tokio::select! {
        result = rx => {
            let _ = result.expect("Failed to receive peer ID");
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(TEST_TIMEOUT)) => {
            panic!("Test timed out");
        }
    }
}

#[tokio::test]
async fn test_get_status() {
    let _server = wait_test_server().await;

    let client = setup_mock_client(&generate_rand_id()).await;

    let _ = client.warm_up_with_delay(WARMUP_TIME).await;

    let (tx, rx) = channel();

    let command = P2pCommand::GetStatus { response_tx: tx };
    client
        .command_tx
        .send(command)
        .await
        .expect("Failed to send command");

    tokio::select! {
        result = rx => {
            let status = result.expect("Failed to receive status");
            match status {
                P2pStatus::NotConnected | P2pStatus::RelayConnected | P2pStatus::PeerConnected(_) => {}
            }
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(TEST_TIMEOUT)) => {
            panic!("Test timed out");
        }
    }
}

#[tokio::test]
async fn test_connect_to_relay() {
    let _server = wait_test_server().await;

    let client = setup_mock_client(&generate_rand_id()).await;
    let _ = client.warm_up_with_delay(WARMUP_TIME).await;

    let (tx, rx) = channel();

    let command = P2pCommand::ConnectToRelay { response_tx: tx };
    client
        .command_tx
        .send(command)
        .await
        .expect("Failed to send command");

    tokio::select! {
        result = rx => {
            let relay_result = result.expect("Failed to receive relay connection status");
            assert!(relay_result.is_ok(), "Relay connection should succeed");
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(TEST_TIMEOUT)) => {
            panic!("Test timed out");
        }
    }
}

#[tokio::test]
async fn test_listening() {
    let _server = wait_test_server().await;

    let client = setup_mock_client(&generate_rand_id()).await;
    let _ = client.warm_up_with_delay(WARMUP_TIME).await;

    let listen_result = client.listen_on_peer(10).await;

    if let Err(e) = listen_result {
        tracing::error!("Failed to start peer listening: {:?}", e);
        panic!("Failed to start peer listening");
    }

    let res = client.is_listening(TEST_TIMEOUT).await;
    match res {
        Ok(_) => {
            tracing::info!("Peer listening started successfully.");
        }
        Err(e) => {
            tracing::error!("Failed to start peer listening: {:?}", e);
            panic!("Failed to start peer listening");
        }
    }
}

#[tokio::test]
async fn test_list_pending_requests() {
    let _server = wait_test_server().await;

    let client = setup_mock_client(&generate_rand_id()).await;
    let _ = client.warm_up_with_delay(WARMUP_TIME).await;

    tokio::fs::write(DUMMY_FILE_PATH, DUMMY_CONTENT)
        .await
        .expect("Failed to create dummy file");

    tokio::select! {
        res = client.send_file_open(DUMMY_FILE_PATH.to_string()) => {
            match res {
                Ok(rx) => {
                    let _ = Some(rx); // receiver for waiting not used for test
                    tracing::info!("File sent open successfully.");
                },
                Err(e) => {
                    tracing::error!("Failed to send file: {:?}", e);
                    panic!("Failed to send file")
                },
            }
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(TEST_TIMEOUT)) => {
            panic!("Test timed out during SendFileOpen");
        }
    }

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let (tx, rx) = tokio::sync::oneshot::channel();
    client
        .command_tx
        .send(P2pCommand::GetPendingRequests { response_tx: tx })
        .await
        .expect("Failed to send GetPendingRequests command");

    tokio::select! {
        res = rx => match res {
            Ok(requests) => {
                tracing::info!("Pending requests: {}", requests.len());
                for req in requests.clone().into_iter() {
                    tracing::info!("  - {:?}", req);
                }
                assert!(
                    requests.contains(&DUMMY_FILE_PATH.to_string()),
                    "The pending request should contain the correct file path"
                );
            }
            Err(e) => {
                tracing::error!("Failed to get pending requests: {}", e);
                panic!("Failed to get pending requests");
            },
        },
        _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {
            panic!("Test timed out while waiting for pending requests");
        }
    }

    tokio::fs::remove_file(DUMMY_FILE_PATH)
        .await
        .expect("Failed to delete dummy file on sender");
}

#[tokio::test]
async fn test_dial_peer() {
    let _server = wait_test_server().await;

    let client_a = setup_mock_client(&generate_rand_id()).await;
    let client_b = setup_mock_client(&generate_rand_id()).await;

    let _ = tokio::join!(
        client_a.warm_up_with_delay(WARMUP_TIME),
        client_b.warm_up_with_delay(WARMUP_TIME)
    );

    // Get B Id
    let peer_id = get_peer_id(&client_b).await;

    // Connect A (dial) -> B (listen)
    for _ in 0..MAX_DIAL_RETRY {
        let (connect_tx, connect_rx) = channel();
        let connect_command = P2pCommand::ConnectToPeer {
            remote_peer_id: PeerId::from_str(&peer_id).expect("Invalid Peer ID"),
            response_tx: connect_tx,
        };
        client_a
            .command_tx
            .send(connect_command)
            .await
            .expect("Failed to send ConnectToPeer command");

        tokio::select! {
            result = connect_rx => {
                let connection_result = result.expect("Failed to receive peer connection status");
                if connection_result.is_ok() {
                    break;
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(TEST_TIMEOUT)) => {
                panic!("Test timed out while connecting to peer");
            }
        }
    }

    // Connection delay
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Check connection status on A
    let (status_tx, status_rx) = channel();
    client_a
        .command_tx
        .send(P2pCommand::GetStatus {
            response_tx: status_tx,
        })
        .await
        .expect("Failed to send GetStatus command");

    tokio::select! {
        result = status_rx => {
            let status = result.expect("Failed to receive peer status");
            match status {
                P2pStatus::PeerConnected(peers) => {
                    tracing::info!("Connected peer ({:?})", peers.len());
                    for peer_id in peers.clone() {
                        tracing::info!("Connected peer: {:?}", peer_id);
                    }
                    assert!(
                        peers.contains(&peer_id),
                        "Connected peer ID should be in the peer list of client_a"
                    );
                }
                _ => panic!("Status should indicate PeerConnected"),
            }
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(TEST_TIMEOUT)) => {
            panic!("Test timed out while fetching connection status");
        }
    }
}

#[tokio::test]
async fn test_receive_file() {
    let _server = wait_test_server().await;

    let client_a = setup_mock_client(&generate_rand_id()).await;
    let client_b = setup_mock_client(&generate_rand_id()).await;

    let _ = tokio::join!(
        client_a.warm_up_with_delay(WARMUP_TIME),
        client_b.warm_up_with_delay(WARMUP_TIME)
    );

    tokio::fs::write(DUMMY_FILE_PATH, DUMMY_CONTENT)
        .await
        .expect("Failed to create dummy file");

    tokio::fs::create_dir_all("./test_dummy")
        .await
        .expect("Failed to create directory for received file");

    let sender_peer_id = get_peer_id(&client_b).await;
    let _ = client_a.connect_peer(sender_peer_id.clone(), 10).await;

    // tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Recv file A <- B
    let (recv_tx, recv_rx) = channel();
    let receive_command = P2pCommand::RecvFile {
        remote_peer_id: sender_peer_id.clone(),
        src_path: DUMMY_FILE_PATH.to_string(),
        tgt_path: DUMMY_RECV_FILE_PATH.to_string(),
        response_tx: recv_tx,
    };
    client_a
        .command_tx
        .send(receive_command)
        .await
        .expect("Failed to send receive command");

    // File transfer completion
    tokio::select! {
        result = recv_rx => {
            let transfer_result = result.expect("Failed to receive file transfer status");
            tracing::error!("Transfer result: {:?}", transfer_result);
            assert!(transfer_result.is_ok(), "File transfer should succeed");
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(TEST_TIMEOUT)) => {
            panic!("Test timed out while receiving file");
        }
    }

    // Verify file size on the receiver's side
    let received_metadata = tokio::fs::metadata(DUMMY_RECV_FILE_PATH)
        .await
        .expect("Failed to get metadata of the received file");
    let expected_size = DUMMY_CONTENT.len() as u64;
    assert_eq!(
        received_metadata.len(),
        expected_size,
        "Received file size should match the expected size"
    );

    // Cleanup
    tokio::fs::remove_file(DUMMY_FILE_PATH)
        .await
        .expect("Failed to delete dummy file on sender");
    tokio::fs::remove_file(DUMMY_RECV_FILE_PATH)
        .await
        .expect("Failed to delete received file on receiver");
    tokio::fs::remove_dir_all("./test_dummy")
        .await
        .expect("Failed to delete test directory");
}

#[tokio::test]
async fn test_integrated_file_transfer() {
    let _server = wait_test_server().await;

    let client_a = setup_mock_client(&generate_rand_id()).await;
    let client_b = setup_mock_client(&generate_rand_id()).await;

    let _ = tokio::join!(
        client_a.warm_up_with_delay(WARMUP_TIME),
        client_b.warm_up_with_delay(WARMUP_TIME)
    );

    tokio::fs::write(INT_DUMMY_FILE_PATH, DUMMY_CONTENT)
        .await
        .expect("Failed to create dummy file");

    tokio::fs::create_dir_all("./test_dummy1")
        .await
        .expect("Failed to create directory for received file");

    let sender_peer_id = get_peer_id(&client_b).await;

    let _ = client_a.connect_peer(sender_peer_id.clone(), 5).await;

    // Start sender
    let mut send_rx: Option<tokio::sync::oneshot::Receiver<Result<(), String>>> = None;
    let sender_future = {
        async move {
            let src_path = INT_DUMMY_FILE_PATH.to_string();
            tokio::select! {
                res = client_b.send_file_open(src_path) => {
                    match res {
                        Ok(rx) => {
                            send_rx = Some(rx);
                            tracing::info!("File sent open successfully.");
                        },
                        Err(e) => tracing::error!("Failed to send file: {}", e),
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(TEST_TIMEOUT)) => {
                    panic!("Test timed out while waiting for receiver to start");
                }
            }

            if let Some(mut rx) = send_rx {
                tokio::select! {
                    res = client_b.send_file_wait(&mut rx, TEST_WAIT_TIMEOUT) => {
                        assert!(res.is_ok(), "File sending should complete successfully");
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_secs(TEST_TIMEOUT)) => {
                        panic!("Test timed out while waiting for file send completion");
                    }
                }
            } else {
                panic!("No send receiver available.");
            }
        }
    };

    // Start receiver
    let receiver_future = {
        async move {
            // Simulate network arrival delay
            tokio::time::sleep(std::time::Duration::from_secs(TEST_RECIVER_TIME_DELAY)).await;

            let (recv_tx, recv_rx) = channel();
            let receive_command = P2pCommand::RecvFile {
                remote_peer_id: sender_peer_id.clone(),
                src_path: INT_DUMMY_FILE_PATH.to_string(),
                tgt_path: INT_DUMMY_RECV_FILE_PATH.to_string(),
                response_tx: recv_tx,
            };

            client_a
                .command_tx
                .send(receive_command)
                .await
                .expect("Failed to send receive command");

            tokio::select! {
                result = recv_rx => {
                    let transfer_result = result.expect("Failed to receive file transfer status");
                    assert!(transfer_result.is_ok(), "File transfer should succeed");
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(TEST_TIMEOUT)) => {
                    panic!("Test timed out while receiving file");
                }
            }
        }
    };

    // Run sender & receiver concurrently
    tokio::join!(sender_future, receiver_future);

    // Verify file size
    let received_metadata = tokio::fs::metadata(INT_DUMMY_RECV_FILE_PATH)
        .await
        .expect("Failed to get metadata of the received file");
    let expected_size = DUMMY_CONTENT.len() as u64;
    assert_eq!(
        received_metadata.len(),
        expected_size,
        "Received file size should match the expected size"
    );

    // Cleanup
    tokio::fs::remove_file(INT_DUMMY_FILE_PATH)
        .await
        .expect("Failed to delete dummy file on sender");
    tokio::fs::remove_file(INT_DUMMY_RECV_FILE_PATH)
        .await
        .expect("Failed to delete received file on receiver");
    tokio::fs::remove_dir_all("./test_dummy1")
        .await
        .expect("Failed to delete test directory");
}

async fn setup_mock_client(client_name: &str) -> P2PTransport {
    let relay_address = LOCAL_RELAY_ADDR;
    let (tx, rx) = tokio::sync::mpsc::channel::<ClientEvent>(1024);
    let res = P2PTransport::new(&relay_address, client_name, tx, PathBuf::from("./"))
        .expect("Failed to create client");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    res
}

async fn get_peer_id(client: &P2PTransport) -> String {
    let (id_tx, id_rx) = channel();
    let get_id_command = P2pCommand::GetId { response_tx: id_tx };
    client
        .command_tx
        .send(get_id_command)
        .await
        .expect("Failed to send GetId command");

    tokio::select! {
        result = id_rx => {
            result.expect("Failed to receive PeerId")
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(TEST_TIMEOUT)) => {
            panic!("Timed out while fetching PeerId");
        }
    }
}

fn generate_rand_id() -> String {
    let suffix: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(3)
        .map(char::from)
        .collect();

    format!("client_{}", suffix)
}
