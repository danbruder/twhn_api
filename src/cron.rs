use crate::result::Result;
use crate::store::Store;
use sqlx::sqlite::SqlitePool;
use tokio::time::{sleep, Duration};

pub async fn start(store: Store, pool: SqlitePool) {
    println!("Starting background work...");

    loop {
        let result = save_rank(&store, &pool).await;
        if result.is_err() {
            println!("Got an error from save rank: {:?}", result);
        }
        sleep(Duration::from_secs(30)).await;
    }
}

async fn save_rank(store: &Store, pool: &SqlitePool) -> Result<()> {
    // Every 30 seconds, get the top list and store the rank of each item
    if let Ok(top_stories) = store.get_top_stories().await {
        println!("Got top stories, saving rank...");

        let mut tx = pool.begin().await?;
        dbg!(&top_stories);

        for (rank, id) in top_stories.into_iter().take(100).enumerate() {
            let rank = (rank + 1) as i32;
            sqlx::query!(
                r#"
            INSERT INTO item_metric (item_id, metric, created_at, value)
            VALUES (?1, 'rank', DATETIME('now'), ?2)
                "#,
                id,
                rank
            )
            .execute(&mut tx)
            .await?;
        }

        println!("Done saving!");
        tx.commit().await?;
    } else {
        println!("couldn't get top stories...");
    }

    Ok(())
}
