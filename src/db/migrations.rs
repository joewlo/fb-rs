use sqlx::migrate::Migrator;
use sqlx::PgPool;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub struct MigrationRunner {
    migrator: &'static Migrator,
}

impl MigrationRunner {
    pub fn new() -> Self {
        Self {
            migrator: &MIGRATOR,
        }
    }

    pub async fn run(&self, pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
        self.migrator.run(pool).await
    }
}

impl Default for MigrationRunner {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    MIGRATOR.run(pool).await
}
