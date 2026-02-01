use std::sync::Arc;

use maedic::{
    configuration::{DBConnectionPool, Settings, SystemState, get_configuration},
    database::setup_database_pool,
    telemetry::initialize_tracing,
};
use once_cell::sync::Lazy;
use sysinfo::System;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub test_user: TestUser,
    pub api_client: reqwest::Client,
    pub pool: DBConnectionPool,
    pub config: Settings,
    pub sys: SystemState,
}

static TRACING: Lazy<()> = Lazy::new(|| {
    initialize_tracing();
});

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c
    };

    setup_database_pool(configuration.database).await;

    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build Application.");
    let application_port = application.port();
    let address = format!("http://127.0.0.1:{}", application.port());
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();
    let _handle = tokio::spawn(application.run_until_stopped());

    let test_app = TestApp {
        pool: setup_database_pool(configuration.database).await,
        port: application_port,
        address,
        api_client: client,
        test_user: TestUser::generate(),
        config: configuration,
        sys: Arc::new(Mutex::new(System::new_all())),
    };
    // test_app.test_user.store(&test_app.pool).await;

    test_app
}

pub struct TestUser {
    user_id: Uuid,
    pub username: String,
    pub password: String,
}
impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            // password: Uuid::new_v4().to_string(),
            password: "everythinghastostartsomewhere".into(),
        }
    }

    // async fn store(&self, pool: &PgPool) {
    //     let salt = SaltString::generate(&mut OsRng);
    //     // Match production parameters
    //     let password_hash = Argon2::new(
    //         Algorithm::Argon2id,
    //         Version::V0x13,
    //         Params::new(15000, 2, 1, None).unwrap(),
    //     )
    //     .hash_password(self.password.as_bytes(), &salt)
    //     .unwrap()
    //     .to_string();
    //
    //     sqlx::query!(
    //         "INSERT INTO users (user_id, username, password_hash)
    //         VALUES ($1, $2, $3)",
    //         self.user_id,
    //         self.username,
    //         password_hash,
    //     )
    //     .execute(pool)
    //     .await
    //     .expect("Failed to store test user.");
    // }
}
