use crate::result::Result;
use crate::store::Store;
use chrono::{DateTime, Utc};
use sqlx::sqlite::SqlitePool;
use tokio::time::{sleep, Duration};

pub async fn start(store: Store, pool: SqlitePool) {
    println!("Starting background work...");

    loop {
        if let Ok(top_stories) = store.get_top_stories().await {
            println!("Got top stories, saving rank...");

            let now = Utc::now();
            let result = save_rank(&pool, top_stories, now).await;
            if result.is_err() {
                println!("Got an error from save rank: {:?}", result);
            }
        }
        sleep(Duration::from_secs(60)).await;
    }
}

async fn save_rank(pool: &SqlitePool, top_stories: Vec<u32>, ts: DateTime<Utc>) -> Result<()> {
    #[derive(Debug)]
    struct ExistingMetric {
        value: i64,
    }

    let mut tx = pool.begin().await?;

    for (rank, id) in top_stories.into_iter().take(100).enumerate() {
        let id = id as i64;
        let rank = (rank + 1) as i64;

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

    println!("Done saving!");
    tx.commit().await?;

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
}
