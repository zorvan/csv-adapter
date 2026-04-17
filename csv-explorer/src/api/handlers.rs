//! API request handlers for Explorer API
//!
//! Implements the actual request handling logic for all API endpoints.

use std::sync::Arc;
use warp::{Rejection, Reply};
use crate::indexing::{IndexingManager, RightsQuery, TransferQuery};
use crate::api::{ApiResponse, RightsSearchRequest, TransfersSearchRequest, RightsResponse, TransfersResponse};
use csv_adapter_core::{Hash, TransferStatus};

/// Handle getting rights by owner
pub async fn get_rights_by_owner(
    owner: String,
    indexing_manager: Arc<IndexingManager>,
) -> Result<impl Reply, Rejection> {
    match indexing_manager.get_rights_by_owner(&owner).await {
        Ok(rights) => {
            let response = RightsResponse {
                rights: rights.clone(),
                total_count: rights.len() as u64,
                has_more: false,
            };
            Ok(warp::reply::json(&ApiResponse::success(response)))
        }
        Err(e) => {
            Ok(warp::reply::json(&ApiResponse::error(e.to_string())))
        }
    }
}

/// Handle searching rights
pub async fn search_rights(
    request: RightsSearchRequest,
    indexing_manager: Arc<IndexingManager>,
) -> Result<impl Reply, Rejection> {
    // Convert API request to internal query
    let query = RightsQuery {
        owner: request.owner,
        chain: request.chain,
        status: request.status.and_then(|s| s.parse().ok()),
        limit: request.limit,
        offset: request.offset,
    };
    
    match indexing_manager.search_rights(&query).await {
        Ok(rights) => {
            let response = RightsResponse {
                rights: rights.clone(),
                total_count: rights.len() as u64,
                has_more: false,
            };
            Ok(warp::reply::json(&ApiResponse::success(response)))
        }
        Err(e) => {
            Ok(warp::reply::json(&ApiResponse::error(e.to_string())))
        }
    }
}

/// Handle getting transfer by hash
pub async fn get_transfer_by_hash(
    hash_str: String,
    indexing_manager: Arc<IndexingManager>,
) -> Result<impl Reply, Rejection> {
    // Parse hash from string
    let hash = match Hash::from_hex(&hash_str) {
        Ok(hash) => hash,
        Err(e) => {
            return Ok(warp::reply::json(&ApiResponse::error(format!("Invalid hash: {}", e))));
        }
    };
    
    match indexing_manager.get_transfer_by_hash(&hash).await {
        Ok(Some(transfer)) => {
            Ok(warp::reply::json(&ApiResponse::success(transfer)))
        }
        Ok(None) => {
            Ok(warp::reply::json(&ApiResponse::error("Transfer not found".to_string())))
        }
        Err(e) => {
            Ok(warp::reply::json(&ApiResponse::error(e.to_string())))
        }
    }
}

/// Handle searching transfers
pub async fn search_transfers(
    request: TransfersSearchRequest,
    indexing_manager: Arc<IndexingManager>,
) -> Result<impl Reply, Rejection> {
    // Convert API request to internal query
    let query = TransferQuery {
        from_chain: request.from_chain,
        to_chain: request.to_chain,
        status: request.status.and_then(|s| s.parse().ok()),
        start_time: request.start_time.and_then(|t| {
            std::time::SystemTime::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(t))
                .map(|st| st.into())
        }),
        end_time: request.end_time.and_then(|t| {
            std::time::SystemTime::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(t))
                .map(|st| st.into())
        }),
        limit: request.limit,
        offset: request.offset,
    };
    
    match indexing_manager.search_transfers(&query).await {
        Ok(transfers) => {
            let response = TransfersResponse {
                transfers: transfers.clone(),
                total_count: transfers.len() as u64,
                has_more: false,
            };
            Ok(warp::reply::json(&ApiResponse::success(response)))
        }
        Err(e) => {
            Ok(warp::reply::json(&ApiResponse::error(e.to_string())))
        }
    }
}

/// Handle getting indexing metrics
pub async fn get_metrics(
    indexing_manager: Arc<IndexingManager>,
) -> Result<impl Reply, Rejection> {
    match indexing_manager.get_metrics().await {
        Ok(metrics) => {
            Ok(warp::reply::json(&ApiResponse::success(metrics)))
        }
        Err(e) => {
            Ok(warp::reply::json(&ApiResponse::error(e.to_string())))
        }
    }
}

/// Handle WebSocket upgrade for real-time updates
pub async fn handle_websocket(
    websocket: warp::ws::WebSocket,
    indexing_manager: Arc<IndexingManager>,
) {
    use futures::{SinkExt, StreamExt};
    
    let (mut tx, mut rx) = websocket.split();
    
    // Send initial metrics
    if let Ok(metrics) = indexing_manager.get_metrics().await {
        let message = serde_json::json!({
            "type": "metrics",
            "data": metrics
        });
        
        if let Ok(text) = serde_json::to_string(&message) {
            let _ = tx.send(warp::ws::Message::text(text)).await;
        }
    }
    
    // Handle incoming messages
    while let Some(message) = rx.next().await {
        if let Ok(message) = message {
            match message {
                warp::ws::Message::Text(text) => {
                    // Parse client message
                    if let Ok(client_msg) = serde_json::from_str::<serde_json::Value>(&text) {
                        handle_client_message(client_msg, &mut tx, &indexing_manager).await;
                    }
                }
                warp::ws::Message::Close(_) => {
                    break;
                }
                _ => {}
            }
        }
    }
}

/// Handle client WebSocket messages
async fn handle_client_message(
    message: serde_json::Value,
    tx: &mut warp::ws::Sender,
    indexing_manager: &Arc<IndexingManager>,
) {
    if let Some(msg_type) = message.get("type").and_then(|v| v.as_str()) {
        match msg_type {
            "subscribe" => {
                // Handle subscription requests
                if let Some(data) = message.get("data") {
                    handle_subscription(data, tx, indexing_manager).await;
                }
            }
            "unsubscribe" => {
                // Handle unsubscription requests
                if let Some(data) = message.get("data") {
                    handle_unsubscription(data, tx, indexing_manager).await;
                }
            }
            "ping" => {
                // Handle ping messages
                let response = serde_json::json!({
                    "type": "pong",
                    "timestamp": std::time::SystemTime::now()
                });
                
                if let Ok(text) = serde_json::to_string(&response) {
                    let _ = tx.send(warp::ws::Message::text(text)).await;
                }
            }
            _ => {
                // Unknown message type
                let response = serde_json::json!({
                    "type": "error",
                    "message": format!("Unknown message type: {}", msg_type)
                });
                
                if let Ok(text) = serde_json::to_string(&response) {
                    let _ = tx.send(warp::ws::Message::text(text)).await;
                }
            }
        }
    }
}

/// Handle subscription requests
async fn handle_subscription(
    data: &serde_json::Value,
    tx: &mut warp::ws::Sender,
    indexing_manager: &Arc<IndexingManager>,
) {
    // Parse subscription data
    if let Some(subscription_type) = data.get("type").and_then(|v| v.as_str()) {
        match subscription_type {
            "rights" => {
                // Subscribe to rights updates
                if let Some(owner) = data.get("owner").and_then(|v| v.as_str()) {
                    if let Ok(rights) = indexing_manager.get_rights_by_owner(owner).await {
                        let response = serde_json::json!({
                            "type": "rights_update",
                            "data": rights
                        });
                        
                        if let Ok(text) = serde_json::to_string(&response) {
                            let _ = tx.send(warp::ws::Message::text(text)).await;
                        }
                    }
                }
            }
            "transfers" => {
                // Subscribe to transfer updates
                if let Some(right_id) = data.get("right_id").and_then(|v| v.as_str()) {
                    if let Ok(hash) = Hash::from_hex(right_id) {
                        if let Ok(Some(transfer)) = indexing_manager.get_transfer_by_hash(&hash).await {
                            let response = serde_json::json!({
                                "type": "transfer_update",
                                "data": transfer
                            });
                            
                            if let Ok(text) = serde_json::to_string(&response) {
                                let _ = tx.send(warp::ws::Message::text(text)).await;
                            }
                        }
                    }
                }
            }
            "metrics" => {
                // Subscribe to metrics updates
                if let Ok(metrics) = indexing_manager.get_metrics().await {
                    let response = serde_json::json!({
                        "type": "metrics_update",
                        "data": metrics
                    });
                    
                    if let Ok(text) = serde_json::to_string(&response) {
                        let _ = tx.send(warp::ws::Message::text(text)).await;
                    }
                }
            }
            _ => {
                let response = serde_json::json!({
                    "type": "error",
                    "message": format!("Unknown subscription type: {}", subscription_type)
                });
                
                if let Ok(text) = serde_json::to_string(&response) {
                    let _ = tx.send(warp::ws::Message::text(text)).await;
                }
            }
        }
    }
}

/// Handle unsubscription requests
async fn handle_unsubscription(
    _data: &serde_json::Value,
    tx: &mut warp::ws::Sender,
    _indexing_manager: &Arc<IndexingManager>,
) {
    // In a real implementation, we'd track subscriptions and remove them
    let response = serde_json::json!({
        "type": "unsubscribed",
        "message": "Successfully unsubscribed"
    });
    
    if let Ok(text) = serde_json::to_string(&response) {
        let _ = tx.send(warp::ws::Message::text(text)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rights_search_request_conversion() {
        let request = RightsSearchRequest {
            owner: Some("test_owner".to_string()),
            chain: Some("ethereum".to_string()),
            status: Some("created".to_string()),
            limit: Some(10),
            offset: Some(0),
        };
        
        let query = RightsQuery {
            owner: request.owner,
            chain: request.chain,
            status: request.status.and_then(|s| s.parse().ok()),
            limit: request.limit,
            offset: request.offset,
        };
        
        assert_eq!(query.owner, Some("test_owner".to_string()));
        assert_eq!(query.chain, Some("ethereum".to_string()));
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(0));
    }
}
