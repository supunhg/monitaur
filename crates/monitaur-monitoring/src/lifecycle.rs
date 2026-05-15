use std::collections::HashMap;

use bollard::Docker;
use bollard::system::EventsOptions;
use futures_util::StreamExt;
use monitaur_core::metrics::LifecycleEvent;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{info, warn};

#[derive(Default)]
pub struct LifecycleTracker;

impl LifecycleTracker {
    pub fn new() -> Self {
        Self
    }

    pub fn start_stream(&self, tx: UnboundedSender<LifecycleEvent>) {
        tokio::spawn(async move {
            let docker = match Docker::connect_with_local_defaults() {
                Ok(d) => d,
                Err(e) => {
                    warn!("Failed to connect to Docker for lifecycle events: {e}");
                    return;
                }
            };

            let options = EventsOptions::<String> {
                since: None,
                until: None,
                filters: HashMap::from([("type".to_string(), vec!["container".to_string()])]),
            };

            let mut stream = docker.events(Some(options));

            while let Some(event_result) = stream.next().await {
                let event = match event_result {
                    Ok(ev) => ev,
                    Err(e) => {
                        warn!("Docker event error: {e}");
                        continue;
                    }
                };

                let action = match &event.action {
                    Some(a) => a.clone(),
                    None => continue,
                };

                let container_id = match event.actor.as_ref().and_then(|a| a.id.clone()) {
                    Some(id) => id,
                    None => continue,
                };

                let lifecycle = match action.as_str() {
                    "start" => {
                        let name = event
                            .actor
                            .as_ref()
                            .and_then(|a| a.attributes.as_ref())
                            .and_then(|attrs| attrs.get("name").cloned())
                            .unwrap_or_default();
                        Some(LifecycleEvent::Started { container_id, name })
                    }
                    "stop" => Some(LifecycleEvent::Stopped { container_id }),
                    "die" => {
                        let exit_code = event
                            .actor
                            .as_ref()
                            .and_then(|a| a.attributes.as_ref())
                            .and_then(|attrs| attrs.get("exitCode"))
                            .and_then(|c| c.parse().ok())
                            .unwrap_or(0);
                        Some(LifecycleEvent::Died {
                            container_id,
                            exit_code,
                        })
                    }
                    "health_status" => {
                        let status = event
                            .actor
                            .as_ref()
                            .and_then(|a| a.attributes.as_ref())
                            .and_then(|attrs| attrs.get("health_status").cloned())
                            .unwrap_or_default();
                        Some(LifecycleEvent::HealthStatus {
                            container_id,
                            status,
                        })
                    }
                    "pause" => Some(LifecycleEvent::Paused { container_id }),
                    "unpause" => Some(LifecycleEvent::Unpaused { container_id }),
                    _ => None,
                };

                if let Some(evt) = lifecycle
                    && tx.send(evt).is_err()
                {
                    break;
                }
            }
        });

        info!("Lifecycle event stream started in background");
    }
}
