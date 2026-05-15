// Docker container enumeration via Docker socket / API

#[derive(Default)]
pub struct DockerDiscoverer;

impl DockerDiscoverer {
    pub fn new() -> Self {
        Self
    }

    pub async fn enumerate_containers(&self) {
        todo!("enumerate containers via Docker API")
    }
}
