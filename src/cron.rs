use crate::result::Result;
use crate::store::Store;
use chrono::{DateTime, Utc};
use sqlx::sqlite::SqlitePool;
use tokio::time::{sleep, Duration};

pub async fn start(store: Store, pool: SqlitePool) {
    println!("Starting background work...");
    let pool_ = pool.clone();
    let store_ = store.clone();

    loop {
        if let Ok(top_stories) = store.get_top_stories().await {
            println!("Got top stories, saving rank...");

            let now = Utc::now();
            let result = save_rank(&pool, top_stories.clone(), now).await;
            if result.is_err() {
                println!("Got an error from save rank: {:?}", result);
            }

            // Cache the items
            let result = store.get_items(top_stories).await;
            if result.is_err() {
                println!("Got an error from loading item and children: {:?}", result);
            }
        }
        sleep(Duration::from_secs(20)).await;

        if let Ok(updates) = store.get_updates().await {
            println!("Got updates");

            let result = store.get_and_store_items(updates.items).await;
            if result.is_err() {
                println!("Got an error from loading updates: {:?}", result);
            }
        }
        sleep(Duration::from_secs(20)).await;
    }
}

async fn save_rank(pool: &SqlitePool, top_stories: Vec<u32>, ts: DateTime<Utc>) -> Result<()> {
    #[derive(Debug)]
    struct ExistingMetric {
        value: i64,
    }

    let mut tx = pool.begin().await?;

    // Delete the old top items
    sqlx::query!("DELETE FROM list WHERE key = 'top_stories'")
        .execute(&mut tx)
        .await?;

    // Save the rank
    for (ordering, id) in top_stories.into_iter().take(30).enumerate() {
        let id = id as i64;
        let ordering = ordering as i64;
        let rank = (ordering + 1) as i64;

        // Save the current list
        sqlx::query!(
            r#"
        INSERT INTO list (key, item_id, ordering, created_at)
        VALUES ('top_stories', ?1, ?2, ?3)
        "#,
            id,
            ordering,
            ts,
        )
        .execute(&mut tx)
        .await?;

        // Get latest value
        let existing = sqlx::query_as!(
            ExistingMetric,
            r#"
                SELECT value FROM item_metric
                WHERE metric = 'rank'
                AND item_id = ?1
                ORDER BY created_at DESC
                LIMIT 1
                "#,
            id,
        )
        .fetch_optional(&mut tx)
        .await?;

        let should_save = match existing {
            Some(ex) if ex.value != rank => true,
            None => true,
            _ => false,
        };

        if should_save {
            sqlx::query!(
                r#"
                    INSERT INTO item_metric (item_id, metric, created_at, value)
                    VALUES (?1, 'rank', ?2, ?3)
                    "#,
                id,
                ts,
                rank
            )
            .execute(&mut tx)
            .await?;
        }
    }

    tx.commit().await?;

    Ok(())
}

async fn backfill_some(pool: &SqlitePool, store: &Store, limit: u32) -> Result<()> {
    let max_item = store.get_max_item_id().await?;

    let start = sqlx::query!("SELECT value FROM config WHERE key='backfill_ptr'")
        .fetch_optional(&*pool)
        .await?
        .map(|v| v.value.parse::<u32>().unwrap())
        .unwrap_or(0);
    let end = (start + limit).min(max_item);

    let range = (start..=end)
        .into_iter()
        .map(|v| v as u32)
        .collect::<Vec<_>>();

    let _ = store.get_and_store_items(range).await;

    sqlx::query!(
        "INSERT OR REPLACE INTO config (key, value) VALUES ('backfill_ptr', ?1)",
        end
    )
    .execute(&*pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::save_rank;
    use chrono::{Duration, Utc};
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
    use std::str::FromStr;

    async fn setup() -> SqlitePool {
        let options = SqliteConnectOptions::from_str("sqlite::memory:").unwrap();
        let pool = SqlitePoolOptions::new().connect_lazy_with(options);
        sqlx::migrate!().run(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn saves_ranks_when_none_exist() {
        let pool = setup().await;

        save_rank(&pool, vec![40], Utc::now()).await.unwrap();

        let got: Vec<(i64, i64)> = sqlx::query_as("SELECT item_id, value FROM item_metric")
            .fetch_all(&pool)
            .await
            .unwrap();

        let want = vec![(40, 1)];
        assert_eq!(got, want)
    }

    #[tokio::test]
    async fn saves_ranks_when_duplicate_exists() {
        let pool = setup().await;
        let t1 = Utc::now() - Duration::seconds(1);
        let t2 = Utc::now();

        save_rank(&pool, vec![40], t1).await.unwrap();
        save_rank(&pool, vec![40], t2).await.unwrap();

        let got: Vec<(i64, i64)> = sqlx::query_as("SELECT item_id, value FROM item_metric")
            .fetch_all(&pool)
            .await
            .unwrap();

        let want = vec![(40, 1)];
        assert_eq!(got, want)
    }

    #[tokio::test]
    async fn saves_ranks_when_duplicate_exists_but_rank_changes() {
        let pool = setup().await;
        let t4 = Utc::now();
        let t3 = t4 - Duration::seconds(1);
        let t2 = t3 - Duration::seconds(1);
        let t1 = t2 - Duration::seconds(1);

        save_rank(&pool, vec![40, 41], t1).await.unwrap();
        save_rank(&pool, vec![40, 41], t2).await.unwrap();
        save_rank(&pool, vec![41, 40], t3).await.unwrap();
        save_rank(&pool, vec![40, 41], t4).await.unwrap();

        let got: Vec<(i64,)> = sqlx::query_as("SELECT  value FROM item_metric WHERE item_id = 40")
            .fetch_all(&pool)
            .await
            .unwrap();

        let want = vec![(1,), (2,), (1,)];
        assert_eq!(got, want)
    }

    #[tokio::test]
    async fn saves_top_stories() {
        let pool = setup().await;

        save_rank(&pool, vec![40], Utc::now()).await.unwrap();

        let got: Vec<(i64, i64)> = sqlx::query_as("SELECT item_id, ordering FROM list")
            .fetch_all(&pool)
            .await
            .unwrap();

        let want = vec![(40, 0)];
        assert_eq!(got, want);

        save_rank(&pool, vec![41], Utc::now()).await.unwrap();

        let got: Vec<(i64, i64)> = sqlx::query_as("SELECT item_id, ordering FROM list")
            .fetch_all(&pool)
            .await
            .unwrap();

        let want = vec![(41, 0)];
        assert_eq!(got, want);
    }
}
