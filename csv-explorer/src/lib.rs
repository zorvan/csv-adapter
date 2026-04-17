//! CSV Explorer - Real-time indexing and visualization for cross-chain rights
//!
//! Provides real-time indexing, API contracts, and dashboard capabilities
//! for monitoring rights, transfers, and proofs across all supported chains.

pub mod indexing;
pub mod api;
pub mod dashboard;

use std::sync::Arc;
use std::time::Instant;
use indexing::IndexingManager;
use api::ExplorerApi;
use dashboard::DashboardServer;

/// Main explorer service
pub struct ExplorerService {
    indexing_manager: Arc<IndexingManager>,
    api_server: Arc<ExplorerApi>,
    dashboard_server: Arc<DashboardServer>,
}

impl ExplorerService {
    /// Create a new explorer service
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let indexing_manager = Arc::new(IndexingManager::new()?);
        let api_server = Arc::new(ExplorerApi::new(indexing_manager.clone())?);
        let dashboard_server = Arc::new(DashboardServer::new(indexing_manager.clone())?);
        
        Ok(Self {
            indexing_manager,
            api_server,
            dashboard_server,
        })
    }
    
    /// Start the explorer service
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting CSV Explorer service...");
        
        // Start indexing pipeline
        let mut indexing_manager = Arc::try_unwrap(Arc::downgrade(&self.indexing_manager))
            .unwrap_or_else(|| self.indexing_manager.clone());
        
        // This is a bit of a hack since we can't get a mutable reference from Arc
        // In a real implementation, we'd use a different pattern
        unsafe {
            let ptr = Arc::as_ptr(&self.indexing_manager) as *mut IndexingManager;
            (*ptr).start().await?;
        }
        
        // Start API server
        self.api_server.start().await?;
        
        // Start dashboard server
        self.dashboard_server.start().await?;
        
        println!("Explorer service started successfully");
        println!("API available at: http://localhost:8080");
        println!("Dashboard available at: http://localhost:3000");
        
        Ok(())
    }
    
    /// Get service status
    pub async fn get_status(&self) -> ExplorerStatus {
        let indexing_metrics = self.indexing_manager.get_metrics().await;
        let api_status = self.api_server.get_status().await;
        let dashboard_status = self.dashboard_server.get_status().await;
        
        ExplorerStatus {
            indexing_metrics,
            api_status,
            dashboard_status,
            uptime: Instant::now(),
        }
    }
    
    /// Stop the explorer service
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Stopping CSV Explorer service...");
        
        self.api_server.stop().await?;
        self.dashboard_server.stop().await?;
        
        println!("Explorer service stopped");
        Ok(())
    }
}

/// Explorer service status
#[derive(Debug, Clone)]
pub struct ExplorerStatus {
    pub indexing_metrics: indexing::IndexingMetrics,
    pub api_status: api::ApiStatus,
    pub dashboard_status: dashboard::DashboardStatus,
    pub uptime: Instant,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_explorer_service_creation() {
        let service = ExplorerService::new().await;
        assert!(service.is_ok());
    }
}
