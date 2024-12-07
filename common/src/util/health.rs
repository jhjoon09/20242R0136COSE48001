use std::time::{Duration, SystemTime};

use tokio::sync::mpsc::{self, Sender};

use crate::{event::Event, message::Message};

pub struct HealthChecker<U: Message, T: Event<U>> {
    sender: Sender<bool>,
    timeout: Duration,
    _marker1: std::marker::PhantomData<T>,
    _marker2: std::marker::PhantomData<U>,
}

impl<U: Message, T: Event<U>> HealthChecker<U, T> {
    fn new(alerter: Sender<T>, alert: T, timeout: Duration) -> Self {
        let (sender, mut receiver) = mpsc::channel::<bool>(1024);

        tokio::spawn(async move {
            let mut timestamp = SystemTime::now();
            while let Some(check) = receiver.recv().await {
                match check {
                    true => {
                        println!("Health check reset: {:?}", timestamp);
                        timestamp = SystemTime::now();
                    }
                    false => {
                        println!("Health check: {:?}", timestamp);

                        if let Ok(elapsed) = timestamp.elapsed() {
                            if elapsed >= timeout {
                                break;
                            }
                        }
                    }
                }
            }
            alerter.send(alert).await
        });

        Self {
            sender,
            timeout,
            _marker1: std::marker::PhantomData,
            _marker2: std::marker::PhantomData,
        }
    }

    pub async fn check(&self) {
        // send health check reset
        self.sender.send(true).await.unwrap();

        // send health check timeout
        let timeout = self.timeout;
        let sender = self.sender.clone();
        tokio::spawn(async move {
            tokio::time::sleep(timeout).await;
            sender.send(false).await
        });
    }

    pub async fn spawn(alerter: Sender<T>, alert: T, timeout: Duration) -> Self {
        let checker = Self::new(alerter, alert, timeout);
        checker.check().await;
        checker
    }
}
