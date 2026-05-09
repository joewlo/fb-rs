use std::collections::HashMap;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::kernel::{AccountResolver, FbError, ResolveContext};

pub struct CachedAccountResolver {
    cache: HashMap<String, Uuid>,
    pool: PgPool,
}

impl CachedAccountResolver {
    pub async fn new(pool: PgPool) -> Result<Self, FbError> {
        let rows = sqlx::query_as::<_, (String, Uuid)>(
            "SELECT account_code, id FROM accounts WHERE status='active'"
        ).fetch_all(&pool).await.map_err(|e| FbError::pipeline("cache", "load", e))?;
        let cache: HashMap<String, Uuid> = rows.into_iter().collect();
        Ok(Self { cache, pool })
    }
}

#[async_trait]
impl AccountResolver for CachedAccountResolver {
    async fn resolve(&self, code_template: &str, _ctx: &ResolveContext) -> Result<Uuid, FbError> {
        if let Some(id) = self.cache.get(code_template) {
            return Ok(*id);
        }
        match sqlx::query_scalar::<_, Uuid>("SELECT id FROM accounts WHERE account_code=$1 AND status='active'")
            .bind(code_template).fetch_optional(&self.pool).await
        {
            Ok(Some(id)) => Ok(id),
            Ok(None) => Err(FbError::AccountNotFound),
            Err(e) => Err(FbError::pipeline("resolve", code_template, e)),
        }
    }
}
