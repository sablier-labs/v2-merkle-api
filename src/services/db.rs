use dotenvy::dotenv;
use migration::{DbErr, Migrator, MigratorTrait};
use sea_orm::{Database, DbConn};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

pub async fn establish_connection() -> Result<Arc<Mutex<DbConn>>, DbErr> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = Database::connect(&database_url)
        .await
        .expect("Failed to setup the database");
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations for tests");

    Ok(Arc::new(Mutex::new(db)))
}

pub fn with_db(
    db_pool: Arc<Mutex<DbConn>>,
) -> impl Filter<Extract = (Arc<Mutex<DbConn>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}
