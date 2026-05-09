use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct Event {
    pub id: Uuid,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub event_type: String,
    pub version: i64,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewEvent {
    pub event_type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, thiserror::Error)]
pub enum EventStoreError {
    #[error("aggregate not found: {0}")]
    NotFound(Uuid),
    #[error("concurrency conflict: expected version {expected}, aggregate at version {actual}")]
    ConcurrencyConflict { expected: i64, actual: i64 },
    #[error("event not found: {0}")]
    EventNotFound(Uuid),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[derive(Debug)]
pub struct PostgresEventStore {
    pool: PgPool,
}

impl PostgresEventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn append(
        &self,
        aggregate_id: Uuid,
        aggregate_type: &str,
        expected_version: Option<i64>,
        events: Vec<NewEvent>,
    ) -> Result<Vec<Event>, EventStoreError> {
        if events.is_empty() {
            return Ok(vec![]);
        }

        let mut tx = self.pool.begin().await?;

        let current_version: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(version) FROM events WHERE aggregate_id = $1",
        )
        .bind(aggregate_id)
        .fetch_one(&mut *tx)
        .await?;

        let current = current_version.unwrap_or(0);

        if let Some(expected) = expected_version {
            if current != expected {
                return Err(EventStoreError::ConcurrencyConflict {
                    expected,
                    actual: current,
                });
            }
        }

        let mut version = current;
        let mut inserted = Vec::with_capacity(events.len());

        for event in &events {
            version += 1;
            let id = Uuid::new_v4();

            sqlx::query(
                "INSERT INTO events (id, aggregate_id, aggregate_type, event_type, version, data) \
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(id)
            .bind(aggregate_id)
            .bind(aggregate_type)
            .bind(&event.event_type)
            .bind(version)
            .bind(&event.data)
            .execute(&mut *tx)
            .await?;

            inserted.push(Event {
                id,
                aggregate_id,
                aggregate_type: aggregate_type.to_string(),
                event_type: event.event_type.clone(),
                version,
                data: event.data.clone(),
                created_at: Utc::now(),
            });
        }

        tx.commit().await?;
        Ok(inserted)
    }

    pub async fn events(
        &self,
        aggregate_id: Option<Uuid>,
        aggregate_type: Option<&str>,
        after_version: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<Event>, EventStoreError> {
        let mut builder = sqlx::QueryBuilder::new(
            "SELECT id, aggregate_id, aggregate_type, event_type, version, data, created_at \
             FROM events WHERE 1=1",
        );

        if let Some(ref id) = aggregate_id {
            builder.push(" AND aggregate_id = ");
            builder.push_bind(id);
        }
        if let Some(ref at) = aggregate_type {
            builder.push(" AND aggregate_type = ");
            builder.push_bind(at);
        }
        if let Some(v) = after_version {
            builder.push(" AND version > ");
            builder.push_bind(v);
        }

        builder.push(" ORDER BY version ASC");

        if let Some(l) = limit {
            builder.push(" LIMIT ");
            builder.push_bind(l);
        }

        let events = builder.build_query_as::<Event>().fetch_all(&self.pool).await?;
        Ok(events)
    }

    pub async fn events_by_aggregate(
        &self,
        aggregate_id: Uuid,
    ) -> Result<Vec<Event>, EventStoreError> {
        let events = sqlx::query_as::<_, Event>(
            "SELECT id, aggregate_id, aggregate_type, event_type, version, data, created_at \
             FROM events WHERE aggregate_id = $1 ORDER BY version ASC",
        )
        .bind(aggregate_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }

    pub async fn latest_version(
        &self,
        aggregate_id: Uuid,
    ) -> Result<Option<i64>, EventStoreError> {
        let version: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(version) FROM events WHERE aggregate_id = $1",
        )
        .bind(aggregate_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(version)
    }

    pub async fn get_by_event_id(
        &self,
        event_id: Uuid,
    ) -> Result<Option<Event>, EventStoreError> {
        let event = sqlx::query_as::<_, Event>(
            "SELECT id, aggregate_id, aggregate_type, event_type, version, data, created_at \
             FROM events WHERE id = $1",
        )
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(event)
    }
}
