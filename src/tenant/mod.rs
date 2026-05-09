use async_trait::async_trait;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
use crate::kernel::{FbError, Tenant};

#[async_trait]
pub trait TenantService: Send + Sync {
    async fn create(&self, name: &str, short_code: &str) -> Result<Tenant, FbError>;
    async fn list(&self) -> Result<Vec<Tenant>, FbError>;
    async fn get_by_code(&self, code: &str) -> Result<Tenant, FbError>;
    async fn activate(&self, code: &str) -> Result<(), FbError>;
    async fn deactivate(&self, code: &str) -> Result<(), FbError>;
}

pub struct PgTenantService {
    pool: PgPool,
}

impl PgTenantService {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl TenantService for PgTenantService {
    async fn create(&self, name: &str, short_code: &str) -> Result<Tenant, FbError> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        sqlx::query("INSERT INTO tenants (id, name, short_code, status, created_at, updated_at) VALUES ($1,$2,$3,'active',$4,$4)")
            .bind(id).bind(name).bind(short_code).bind(now)
            .execute(&self.pool).await.map_err(|e| FbError::PipelineError { stage: "tenant".into(), tx_id: short_code.into(), reason: e.to_string() })?;

        Ok(Tenant { id, name: name.into(), short_code: short_code.into(), status: "active".into(), metadata: None, created_at: now, updated_at: now })
    }

    async fn list(&self) -> Result<Vec<Tenant>, FbError> {
        let rows = sqlx::query_as::<_, (Uuid, String, String, String, chrono::DateTime<Utc>, chrono::DateTime<Utc>)>(
            "SELECT id, name, short_code, status, created_at, updated_at FROM tenants ORDER BY name"
        ).fetch_all(&self.pool).await.map_err(|e| FbError::PipelineError { stage: "tenant".into(), tx_id: "".into(), reason: e.to_string() })?;

        Ok(rows.into_iter().map(|(id, name, short_code, status, ca, ua)| Tenant { id, name, short_code, status, metadata: None, created_at: ca, updated_at: ua }).collect())
    }

    async fn get_by_code(&self, code: &str) -> Result<Tenant, FbError> {
        let row = sqlx::query_as::<_, (Uuid, String, String, String, chrono::DateTime<Utc>, chrono::DateTime<Utc>)>(
            "SELECT id, name, short_code, status, created_at, updated_at FROM tenants WHERE short_code=$1"
        ).bind(code).fetch_optional(&self.pool).await.map_err(|e| FbError::PipelineError { stage: "tenant".into(), tx_id: code.into(), reason: e.to_string() })?;

        match row {
            Some((id, name, short_code, status, ca, ua)) => Ok(Tenant { id, name, short_code, status, metadata: None, created_at: ca, updated_at: ua }),
            None => Err(FbError::TenantNotFound),
        }
    }

    async fn activate(&self, code: &str) -> Result<(), FbError> {
        let r = sqlx::query("UPDATE tenants SET status='active', updated_at=NOW() WHERE short_code=$1").bind(code).execute(&self.pool).await.map_err(|e| FbError::PipelineError { stage: "tenant".into(), tx_id: code.into(), reason: e.to_string() })?;
        if r.rows_affected() == 0 { return Err(FbError::TenantNotFound); }
        Ok(())
    }

    async fn deactivate(&self, code: &str) -> Result<(), FbError> {
        let r = sqlx::query("UPDATE tenants SET status='inactive', updated_at=NOW() WHERE short_code=$1").bind(code).execute(&self.pool).await.map_err(|e| FbError::PipelineError { stage: "tenant".into(), tx_id: code.into(), reason: e.to_string() })?;
        if r.rows_affected() == 0 { return Err(FbError::TenantNotFound); }
        Ok(())
    }
}
