/// Configuración del servidor cargada desde variables de entorno.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub google_client_id: String,
    pub server_host: String,
    pub server_port: u16,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            google_client_id: std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default(),
            server_host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .expect("SERVER_PORT must be a number"),
        }
    }
}
