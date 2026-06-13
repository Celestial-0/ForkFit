use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use uuid::Uuid;
use crate::app::AppState;
use crate::common::id::{UserId, NotificationLogId};
use crate::background::models::{BackgroundJob, NotificationLog};
use crate::background::repository::BackgroundRepository;
use crate::infra::pg::background_repo::PgBackgroundRepository;

pub async fn run_background_jobs(state: Arc<AppState>) {
    let worker_id = Uuid::new_v4();
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    let repo = PgBackgroundRepository::new(state.db.clone());

    tracing::info!(%worker_id, "background_job_runner_started");

    loop {
        interval.tick().await;

        // Fetch up to 5 processing jobs locked under this worker
        let jobs = match repo.get_next_jobs("default", 5, worker_id).await {
            Ok(j) => j,
            Err(e) => {
                tracing::error!(error = %e, "failed_to_fetch_background_jobs");
                continue;
            }
        };

        for job in jobs {
            let state_clone = state.clone();
            let repo_clone = repo.clone();
            tokio::spawn(async move {
                let job_id = job.id;
                let job_type = job.job_type.clone();
                tracing::info!(%job_id, %job_type, "starting_background_job");

                match process_job(state_clone, &repo_clone, &job).await {
                    Ok(_) => {
                        if let Err(e) = repo_clone.mark_job_completed(job_id).await {
                            tracing::error!(%job_id, error = %e, "failed_to_mark_job_completed");
                        } else {
                            tracing::info!(%job_id, %job_type, "completed_background_job");
                        }
                    }
                    Err(e) => {
                        tracing::error!(%job_id, %job_type, error = %e, "failed_background_job");
                        if let Err(err) = repo_clone.mark_job_failed(job_id, &e.to_string(), None).await {
                            tracing::error!(%job_id, error = %err, "failed_to_mark_job_failed");
                        }
                    }
                }
            });
        }
    }
}

async fn process_job(
    state: Arc<AppState>,
    repo: &PgBackgroundRepository,
    job: &BackgroundJob,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match job.job_type.as_str() {
        "send_email" => {
            // payload: { "recipient": "...", "subject": "...", "body": "...", "user_id": "..." }
            let payload = &job.payload;
            let recipient = payload["recipient"].as_str().ok_or("missing recipient")?;
            let subject = payload["subject"].as_str().ok_or("missing subject")?;
            let body = payload["body"].as_str().ok_or("missing body")?;
            let user_id_str = payload["user_id"].as_str();
            
            let user_id = if let Some(uid_str) = user_id_str {
                Uuid::parse_str(uid_str).ok().map(UserId)
            } else {
                None
            };

            // Call mailer
            state.mailer.send_email(recipient, subject, body).await?;

            // Insert notification log
            let notif_log = NotificationLog {
                id: NotificationLogId::new(),
                user_id,
                channel: "email".to_string(),
                recipient: recipient.to_string(),
                status: "sent".to_string(),
                provider: Some("console_smtp".to_string()),
                provider_message_id: Some(Uuid::new_v4().to_string()),
                error_message: None,
                sent_at: Some(Utc::now()),
                created_at: Utc::now(),
            };
            repo.log_notification(notif_log).await?;
            Ok(())
        }
        "cleanup_sessions" => {
            sqlx::query("DELETE FROM sessions WHERE expires_at <= NOW() OR revoked_at IS NOT NULL")
                .execute(&state.db)
                .await?;
            Ok(())
        }
        "weekly_report" => {
            // payload: { "user_id": "..." }
            let payload = &job.payload;
            let user_id_str = payload["user_id"].as_str().ok_or("missing user_id")?;
            let user_uuid = Uuid::parse_str(user_id_str)?;
            let user_id = UserId(user_uuid);

            // 1. Get user email
            let email = sqlx::query_scalar::<_, String>("SELECT email FROM users WHERE id = $1")
                .bind(user_uuid)
                .fetch_one(&state.db)
                .await?;

            // 2. Query target nutrition goal
            let target_row = sqlx::query!(
                r#"
                SELECT target_value::float8 as "target_value!", config
                FROM user_goals
                WHERE user_id = $1 AND category = 'nutrition' AND is_active = true
                LIMIT 1
                "#,
                user_uuid
            )
            .fetch_optional(&state.db)
            .await?;

            let target_calories = target_row.as_ref().map(|r| r.target_value).unwrap_or(2000.0);
            let target_config = target_row.as_ref().map(|r| r.config.clone()).unwrap_or(serde_json::json!({}));

            // 3. Query average food logs for last 7 days
            let log_summary = sqlx::query!(
                r#"
                SELECT 
                    COALESCE(SUM(calories), 0.00)::float8 as "total_calories!",
                    COALESCE(SUM(protein), 0.00)::float8 as "total_protein!",
                    COALESCE(SUM(carbs), 0.00)::float8 as "total_carbs!",
                    COALESCE(SUM(fats), 0.00)::float8 as "total_fats!"
                FROM food_logs
                WHERE user_id = $1 AND logged_at >= NOW() - INTERVAL '7 days'
                "#,
                user_uuid
            )
            .fetch_one(&state.db)
            .await?;

            // Divide by 7 to get daily average
            let avg_cal = log_summary.total_calories / 7.0;
            let avg_prot = log_summary.total_protein / 7.0;
            let avg_carbs = log_summary.total_carbs / 7.0;
            let avg_fats = log_summary.total_fats / 7.0;

            let report_body = format!(
                "Weekly Nutrition Summary:\n\n\
                 Your daily averages over the last 7 days:\n\
                 - Calories: {:.1} kcal (Target: {:.1} kcal)\n\
                 - Protein: {:.1}g (Target: {:.1}g)\n\
                 - Carbs: {:.1}g (Target: {:.1}g)\n\
                 - Fats: {:.1}g (Target: {:.1}g)\n\n\
                 Keep up the great work!",
                avg_cal,
                target_calories,
                avg_prot,
                target_config["protein_g"].as_f64().unwrap_or(150.0),
                avg_carbs,
                target_config["carbs_g"].as_f64().unwrap_or(200.0),
                avg_fats,
                target_config["fats_g"].as_f64().unwrap_or(60.0)
            );

            // Call mailer
            state.mailer.send_email(&email, "Your Weekly ForkFit Nutrition Report", &report_body).await?;

            // Insert notification log
            let notif_log = NotificationLog {
                id: NotificationLogId::new(),
                user_id: Some(user_id),
                channel: "email".to_string(),
                recipient: email,
                status: "sent".to_string(),
                provider: Some("console_smtp".to_string()),
                provider_message_id: Some(Uuid::new_v4().to_string()),
                error_message: None,
                sent_at: Some(Utc::now()),
                created_at: Utc::now(),
            };
            repo.log_notification(notif_log).await?;
            Ok(())
        }
        "generate_embeddings" => {
            // Find recipes without embeddings
            let unindexed_recipes = sqlx::query!(
                r#"
                SELECT id, title, description 
                FROM recipes
                WHERE id NOT IN (SELECT recipe_id FROM recipe_embeddings)
                LIMIT 5
                "#
            )
            .fetch_all(&state.db)
            .await?;

            for rec in unindexed_recipes {
                let chunk_text = format!(
                    "Recipe: {}\nDescription: {}", 
                    rec.title, 
                    rec.description.unwrap_or_default()
                );

                // Generate deterministic vector of 1536 floats for testing/mock purposes
                let mut vec_str = String::with_capacity(30 * 1536);
                vec_str.push('[');
                for i in 0..1536 {
                    if i > 0 {
                        vec_str.push(',');
                    }
                    // Deterministic value based on title characters
                    let val = (((rec.title.len() + i) % 100) as f32) / 100.0;
                    vec_str.push_str(&format!("{:.4}", val));
                }
                vec_str.push(']');

                sqlx::query(
                    r#"
                    INSERT INTO recipe_embeddings (recipe_id, embedding, chunk_text, updated_at)
                    VALUES ($1, $2::vector, $3, NOW())
                    ON CONFLICT (recipe_id) DO UPDATE
                    SET embedding = EXCLUDED.embedding,
                        chunk_text = EXCLUDED.chunk_text,
                        updated_at = NOW()
                    "#,
                )
                .bind(rec.id)
                .bind(&vec_str)
                .bind(chunk_text)
                .execute(&state.db)
                .await?;

                tracing::info!(recipe_id = ?rec.id, "generated_recipe_embedding");
            }
            Ok(())
        }
        _ => Err(format!("unknown job type: {}", job.job_type).into()),
    }
}
