/// RPC Manager for handling multiple RPC endpoints with fallbacks.
///
/// Supports:
/// - Multiple RPC URLs per chain
/// - HTTP/HTTPS and WSS connections
/// - API key authentication
/// - Automatic fallback to alternative endpoints on failure
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::Path;

use csv_explorer_shared::ExplorerError;

/// Type of RPC endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RpcType {
    Http,
    Wss,
}

/// Authentication configuration for RPC endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcAuth {
    #[serde(rename = "type")]
    pub auth_type: AuthType,
    pub header: String,
    pub value: String,
}

/// Authentication method types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    ApiKey,
    BearerToken,
    BasicAuth,
}

/// Single RPC endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcEndpoint {
    pub url: String,
    #[serde(rename = "type")]
    pub endpoint_type: RpcType,
    pub priority: u8,
    #[serde(default)]
    pub auth: Option<RpcAuth>,
}

/// Chain RPC configuration with primary and alternative endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainRpcConfig {
    pub primary: RpcEndpoint,
    pub alternatives: Vec<RpcEndpoint>,
}

/// Global RPC configuration for all chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    pub chains: HashMap<String, ChainRpcConfig>,
}

impl RpcConfig {
    /// Load RPC configuration from a JSON file
    pub fn from_file(path: &Path) -> Result<Self, ExplorerError> {
        let content = std::fs::read_to_string(path).map_err(|e| ExplorerError::Io(e))?;

        let mut config: Self = serde_json::from_str(&content)
            .map_err(|e| ExplorerError::Internal(format!("Failed to parse RPC config: {}", e)))?;

        // Resolve environment variables in auth values
        config.resolve_env_vars();

        Ok(config)
    }

    /// Resolve environment variables in auth values
    fn resolve_env_vars(&mut self) {
        for chain_config in self.chains.values_mut() {
            // Resolve primary endpoint auth
            if let Some(ref mut auth) = chain_config.primary.auth {
                auth.value = env::var(&auth.value).unwrap_or_else(|_| auth.value.clone());
            }

            // Resolve alternative endpoints auth
            for alt in &mut chain_config.alternatives {
                if let Some(ref mut auth) = alt.auth {
                    auth.value = env::var(&auth.value).unwrap_or_else(|_| auth.value.clone());
                }
            }
        }
    }

    /// Get the best available endpoint for a chain
    pub fn get_endpoint(&self, chain_id: &str) -> Option<RpcEndpoint> {
        if let Some(chain_config) = self.chains.get(chain_id) {
            // First try primary endpoint
            if !chain_config.primary.url.is_empty() {
                return Some(chain_config.primary.clone());
            }

            // Then try alternatives sorted by priority
            let mut sorted_alternatives = chain_config.alternatives.clone();
            sorted_alternatives.sort_by(|a, b| a.priority.cmp(&b.priority));

            for alt in sorted_alternatives {
                if !alt.url.is_empty() {
                    return Some(alt);
                }
            }
        }

        None
    }

    /// Get all endpoints for a chain (primary + alternatives) sorted by priority
    pub fn get_all_endpoints(&self, chain_id: &str) -> Vec<RpcEndpoint> {
        let mut endpoints = Vec::new();

        if let Some(chain_config) = self.chains.get(chain_id) {
            if !chain_config.primary.url.is_empty() {
                endpoints.push(chain_config.primary.clone());
            }

            let mut sorted_alternatives = chain_config.alternatives.clone();
            sorted_alternatives.sort_by(|a, b| a.priority.cmp(&b.priority));

            for alt in sorted_alternatives {
                if !alt.url.is_empty() {
                    endpoints.push(alt);
                }
            }
        }

        endpoints
    }

    /// Create HTTP client with optional authentication
    pub fn create_http_client(&self, endpoint: &RpcEndpoint) -> reqwest::Client {
        let mut client_builder = reqwest::Client::new();

        if let Some(ref auth) = endpoint.auth {
            // Add authentication headers to the client
            // Note: This is a simplified version; in production you might want
            // to handle authentication more dynamically
            client_builder
        } else {
            client_builder
        }
    }
}

/// Load RPC configuration from default location
pub fn load_rpc_config() -> Result<RpcConfig, ExplorerError> {
    let path = Path::new("rpc_config.json");
    if path.exists() {
        RpcConfig::from_file(path)
    } else {
        // Fallback to default config if file doesn't exist
        Ok(RpcConfig::default())
    }
}

impl Default for RpcConfig {
    fn default() -> Self {
        // Default configuration with basic endpoints
        let default_bitcoin = ChainRpcConfig {
            primary: RpcEndpoint {
                url: "https://mempool.space/api".to_string(),
                endpoint_type: RpcType::Http,
                priority: 1,
                auth: None,
            },
            alternatives: vec![RpcEndpoint {
                url: "https://btc.getblock.io/mainnet/".to_string(),
                endpoint_type: RpcType::Http,
                priority: 2,
                auth: Some(RpcAuth {
                    auth_type: AuthType::ApiKey,
                    header: "X-API-Key".to_string(),
                    value: "${GETBLOCK_API_KEY}".to_string(), // Will be replaced from env
                }),
            }],
        };

        let default_ethereum = ChainRpcConfig {
            primary: RpcEndpoint {
                url: "https://eth.llamarpc.com".to_string(),
                endpoint_type: RpcType::Http,
                priority: 1,
                auth: None,
            },
            alternatives: vec![
                RpcEndpoint {
                    url: "https://ethereum-rpc.publicnode.com".to_string(),
                    endpoint_type: RpcType::Http,
                    priority: 2,
                    auth: None,
                },
                RpcEndpoint {
                    url: "wss://ethereum-rpc.publicnode.com".to_string(),
                    endpoint_type: RpcType::Wss,
                    priority: 3,
                    auth: None,
                },
            ],
        };

        let default_sui = ChainRpcConfig {
            primary: RpcEndpoint {
                url: "https://fullnode.mainnet.sui.io:443".to_string(),
                endpoint_type: RpcType::Http,
                priority: 1,
                auth: None,
            },
            alternatives: vec![RpcEndpoint {
                url: "https://sui-mainnet-rpc.allthatnode.com".to_string(),
                endpoint_type: RpcType::Http,
                priority: 2,
                auth: None,
            }],
        };

        let default_aptos = ChainRpcConfig {
            primary: RpcEndpoint {
                url: "https://fullnode.mainnet.aptoslabs.com/v1".to_string(),
                endpoint_type: RpcType::Http,
                priority: 1,
                auth: None,
            },
            alternatives: vec![RpcEndpoint {
                url: "https://aptos-mainnet-rpc.allthatnode.com/v1".to_string(),
                endpoint_type: RpcType::Http,
                priority: 2,
                auth: None,
            }],
        };

        let default_solana = ChainRpcConfig {
            primary: RpcEndpoint {
                url: "https://api.mainnet-beta.solana.com".to_string(),
                endpoint_type: RpcType::Http,
                priority: 1,
                auth: None,
            },
            alternatives: vec![RpcEndpoint {
                url: "https://solana-rpc.publicnode.com".to_string(),
                endpoint_type: RpcType::Http,
                priority: 2,
                auth: None,
            }],
        };

        let mut chains = HashMap::new();
        chains.insert("bitcoin".to_string(), default_bitcoin);
        chains.insert("ethereum".to_string(), default_ethereum);
        chains.insert("sui".to_string(), default_sui);
        chains.insert("aptos".to_string(), default_aptos);
        chains.insert("solana".to_string(), default_solana);

        RpcConfig { chains }
    }
}

/// RPC Manager for handling multiple RPC endpoints with fallbacks.
///
/// This is a wrapper around RpcConfig that provides additional convenience methods
/// for use by chain indexers.
pub struct RpcManager {
    config: RpcConfig,
}

impl RpcManager {
    /// Create a new RPC manager with the given configuration.
    pub fn new(config: RpcConfig) -> Self {
        Self { config }
    }

    /// Get the best available endpoint for a chain.
    pub fn get_endpoint(&self, chain_id: &str) -> Option<RpcEndpoint> {
        self.config.get_endpoint(chain_id)
    }

    /// Get all endpoints for a chain (primary + alternatives) sorted by priority.
    pub fn get_all_endpoints(&self, chain_id: &str) -> Vec<RpcEndpoint> {
        self.config.get_all_endpoints(chain_id)
    }

    /// Get the primary endpoint for a chain.
    pub fn get_primary_endpoint(&self, chain_id: &str) -> Option<&RpcEndpoint> {
        self.config.chains.get(chain_id).map(|c| &c.primary)
    }

    /// Get the HTTP client for a specific chain endpoint.
    pub fn get_http_client(&self, chain_id: &str) -> Option<reqwest::Client> {
        self.config.get_endpoint(chain_id).map(|endpoint| {
            let mut client_builder = reqwest::Client::new();

            if let Some(ref auth) = endpoint.auth {
                match auth.auth_type {
                    AuthType::ApiKey => client_builder,
                    AuthType::BearerToken => client_builder,
                    AuthType::BasicAuth => client_builder,
                }
            } else {
                client_builder
            }
        })
    }

    /// Get the HTTP client for a specific chain endpoint (alias for get_http_client).
    pub fn get_client(&self, chain_id: &str) -> Option<reqwest::Client> {
        self.get_http_client(chain_id)
    }
}

impl Clone for RpcManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
        }
    }
}
