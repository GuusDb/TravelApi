use actix_web::{web, App, HttpServer, middleware::Logger};
use dotenv::dotenv;
use log::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod api_docs;
mod db;
mod middleware;
mod models;
mod routes;
mod services;

use crate::api_docs::ApiDoc;
use crate::db::connection;
use crate::routes::{auth, travel_plan, route_option};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    let db_pool = match connection::get_pool() {
        Ok(pool) => {
            info!("Database connection pool created successfully");
            pool
        },
        Err(e) => {
            panic!("Failed to create database connection pool: {}", e);
        }
    };
    
    let db_data = web::Data::new(db_pool);
    
    info!("Starting HTTP server at http://127.0.0.1:8080");
    
    HttpServer::new(move || {
        let openapi = ApiDoc::openapi();
        
        App::new()
            .wrap(Logger::default())
            
            .app_data(db_data.clone())
            
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone())
            )
            
            .service(
                web::scope("/api")
                    .route("/register", web::post().to(auth::register))
                    .route("/login", web::post().to(auth::login))
                    
                    .route("/travelplan", web::get().to(travel_plan::get_travel_plans))
                    .route("/travelplan", web::post().to(travel_plan::create_travel_plan))
                    .route("/travelplan/{id}", web::get().to(travel_plan::get_travel_plan_by_id))
                    .route("/travelplan/{id}", web::put().to(travel_plan::update_travel_plan))
                    .route("/travelplan/{id}", web::delete().to(travel_plan::delete_travel_plan))
                    
                    .route("/travelplan/{id}/routes", web::get().to(route_option::get_route_options))
                    .route("/travelplan/{id}/routes/generate", web::post().to(route_option::generate_route_options))
                    .route("/travelplan/{plan_id}/routes/{route_id}", web::get().to(route_option::get_route_option_by_id))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
