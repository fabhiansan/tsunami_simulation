use std::env;
use tsunami_simulation::api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Get port from environment variable or use default
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);
    
    println!("Starting Tsunami Simulation API server on port {}", port);
    
    // Start API server
    api::start_api_server(port).await
}
