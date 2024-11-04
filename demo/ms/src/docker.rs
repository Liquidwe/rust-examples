use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug)]
pub struct DockerManager {
    image: String,
}

impl DockerManager {
    pub fn new() -> Self {
        Self {
            image: "repository.chainbase.com/manuscript-node/manuscript-debug:v0.0.1".to_string(),
        }
    }

    pub async fn setup(&self) -> Result<String, String> {
        // Step 1: Check if docker is installed
        if !self.check_docker_installed() {
            return Err("Docker is not installed or not accessible".to_string());
        }

        // Step 2: Pull the image
        match self.pull_image().await {
            Ok(_) => println!("Image pulled successfully"),
            Err(e) => return Err(format!("Failed to pull image: {}", e)),
        }

        // Step 3: Run the container
        match self.run_container().await {
            Ok(_) => Ok("Container started successfully".to_string()),
            Err(e) => Err(format!("Failed to start container: {}", e)),
        }
    }

    pub fn get_setup_progress(&self, timer: u64) -> Vec<(String, bool)> {
        let seconds = timer as f64 / 10.0;
        vec![
            ("Checking Docker installation...".to_string(), seconds >= 0.0),
            ("Pulling required image...".to_string(), seconds >= 2.0),
            ("Starting container...".to_string(), seconds >= 5.0),
        ]
    }

    fn check_docker_installed(&self) -> bool {
        Command::new("docker")
            .arg("--version")
            .output()
            .is_ok()
    }

    async fn pull_image(&self) -> Result<(), String> {
        let output = Command::new("docker")
            .args(["pull", &self.image])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    async fn run_container(&self) -> Result<(), String> {
        let output = Command::new("docker")
            .args([
                "run",
                "-d",  // Run in detached mode
                "--rm",
                "-p", "8083:8083",
                "-p", "8081:8081",
                &self.image,
            ])
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        // Wait for container to be ready (you might want to implement a proper health check)
        sleep(Duration::from_secs(5)).await;
        Ok(())
    }
}

impl Clone for DockerManager {
    fn clone(&self) -> Self {
        Self::new()
    }
} 