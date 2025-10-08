use neo4rs::{Graph, ConfigBuilder, Config};


/// Connects to Neo4j using hardcoded credentials 
pub async fn connect(config: Config) -> String {
    //SINK
    match Graph::connect(config).await {
        Ok(_) => "Vulnerable: Connected to Neo4j with hardcoded credentials".to_string(),
        Err(e) => format!("Vulnerable: Failed to connect - {}", e),
    }
}