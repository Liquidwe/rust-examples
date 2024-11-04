use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;
use tokio::sync::mpsc;
use crate::app::AppUpdate;

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

    pub async fn setup(&self, sender: Option<mpsc::Sender<AppUpdate>>) -> Result<String, String> {
        // Step 1: Check if docker is installed
        if !self.check_docker_installed() {
            return Err("Docker is not installed or not accessible".to_string());
        }

        if let Some(sender) = &sender {
            let _ = sender.send(AppUpdate::DockerStatus(
                "Checking Docker installation...".to_string()
            )).await;
        }

        // Step 2: Pull the image
        if let Some(sender) = &sender {
            let _ = sender.send(AppUpdate::DockerStatus(
                "Pulling required images...".to_string()
            )).await;
        }
        
        if let Err(e) = self.pull_image().await {
            return Err(format!("Failed to pull image: {}", e));
        }

        // Step 3: Run the container
        if let Some(sender) = &sender {
            let _ = sender.send(AppUpdate::DockerStatus(
                "Starting container...".to_string()
            )).await;
        }
        
        if let Err(e) = self.run_container().await {
            return Err(format!("Failed to start container: {}", e));
        }
        
        Ok("Container started successfully".to_string())
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
                "-p", "18083:8083",
                "-p", "18081:8081",
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

    pub fn get_setup_progress(&self, timer: u64) -> Vec<(String, bool)> {
        vec![
            ("Checking Docker installation".to_string(), timer >= 10),
            ("Pulling required images".to_string(), timer >= 30),
            ("Starting containers".to_string(), timer >= 50),
            ("Configuring network".to_string(), timer >= 70),
            ("Verifying setup".to_string(), timer >= 90)
        ]
    }
}

impl Clone for DockerManager {
    fn clone(&self) -> Self {
        Self::new()
    }
} 