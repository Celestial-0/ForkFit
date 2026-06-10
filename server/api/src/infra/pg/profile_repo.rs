use sqlx::{PgPool, query_as, query, query_scalar};
use uuid::Uuid;

use crate::common::AppResult;
use crate::common::id::{UserId, GoalId, BiometricLogId, WorkoutLogId};
use crate::profile::models::{
    Profile, UserPreference, MedicalSafetyProfile, BiometricLog, WorkoutLog, UserGoal,
};
use crate::profile::repository::ProfileRepository;

#[derive(Clone)]
pub struct PgProfileRepository {
    pool: PgPool,
}

impl PgProfileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ProfileRepository for PgProfileRepository {
    async fn get_profile(&self, user_id: UserId) -> AppResult<Option<Profile>> {
        let profile = query_as!(
            Profile,
            r#"
            SELECT
                user_id as "user_id: UserId",
                full_name,
                avatar_url,
                gender,
                dob,
                timezone as "timezone!",
                created_at,
                updated_at
            FROM profiles
            WHERE user_id = $1
            "#,
            user_id as UserId
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(profile)
    }

    async fn upsert_profile(&self, profile: Profile) -> AppResult<Profile> {
        let res = query_as!(
            Profile,
            r#"
            INSERT INTO profiles (user_id, full_name, avatar_url, gender, dob, timezone, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, now())
            ON CONFLICT (user_id) DO UPDATE SET
                full_name = EXCLUDED.full_name,
                avatar_url = EXCLUDED.avatar_url,
                gender = EXCLUDED.gender,
                dob = EXCLUDED.dob,
                timezone = EXCLUDED.timezone,
                updated_at = now()
            RETURNING
                user_id as "user_id: UserId",
                full_name,
                avatar_url,
                gender,
                dob,
                timezone as "timezone!",
                created_at,
                updated_at
            "#,
            profile.user_id as UserId,
            profile.full_name,
            profile.avatar_url,
            profile.gender,
            profile.dob,
            profile.timezone
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res)
    }

    async fn get_preferences(&self, user_id: UserId) -> AppResult<Option<UserPreference>> {
        let prefs = query_as!(
            UserPreference,
            r#"
            SELECT
                user_id as "user_id: UserId",
                theme,
                language,
                measurement_system,
                preferences,
                created_at,
                updated_at
            FROM user_preferences
            WHERE user_id = $1
            "#,
            user_id as UserId
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(prefs)
    }

    async fn upsert_preferences(&self, prefs: UserPreference) -> AppResult<UserPreference> {
        let res = query_as!(
            UserPreference,
            r#"
            INSERT INTO user_preferences (user_id, theme, language, measurement_system, preferences, updated_at)
            VALUES ($1, $2, $3, $4, $5, now())
            ON CONFLICT (user_id) DO UPDATE SET
                theme = EXCLUDED.theme,
                language = EXCLUDED.language,
                measurement_system = EXCLUDED.measurement_system,
                preferences = EXCLUDED.preferences,
                updated_at = now()
            RETURNING
                user_id as "user_id: UserId",
                theme,
                language,
                measurement_system,
                preferences,
                created_at,
                updated_at
            "#,
            prefs.user_id as UserId,
            prefs.theme,
            prefs.language,
            prefs.measurement_system,
            prefs.preferences
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res)
    }

    async fn get_medical_safety(&self, user_id: UserId) -> AppResult<Option<MedicalSafetyProfile>> {
        let profile = query_as!(
            MedicalSafetyProfile,
            r#"
            SELECT
                user_id as "user_id: UserId",
                allergies,
                medical_conditions,
                is_pregnant,
                is_lactating,
                updated_at
            FROM medical_safety_profiles
            WHERE user_id = $1
            "#,
            user_id as UserId
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(profile)
    }

    async fn upsert_medical_safety(&self, profile: MedicalSafetyProfile) -> AppResult<MedicalSafetyProfile> {
        let res = query_as!(
            MedicalSafetyProfile,
            r#"
            INSERT INTO medical_safety_profiles (user_id, allergies, medical_conditions, is_pregnant, is_lactating, updated_at)
            VALUES ($1, $2, $3, $4, $5, now())
            ON CONFLICT (user_id) DO UPDATE SET
                allergies = EXCLUDED.allergies,
                medical_conditions = EXCLUDED.medical_conditions,
                is_pregnant = EXCLUDED.is_pregnant,
                is_lactating = EXCLUDED.is_lactating,
                updated_at = now()
            RETURNING
                user_id as "user_id: UserId",
                allergies,
                medical_conditions,
                is_pregnant,
                is_lactating,
                updated_at
            "#,
            profile.user_id as UserId,
            &profile.allergies,
            &profile.medical_conditions,
            profile.is_pregnant,
            profile.is_lactating
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(res)
    }

    async fn log_biometric(&self, log: BiometricLog) -> AppResult<BiometricLog> {
        let raw_id: Uuid = log.id.into();
        let res = query!(
            r#"
            INSERT INTO biometric_logs (id, user_id, logged_at, metric_type, value, notes)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING
                id as "id: BiometricLogId",
                user_id as "user_id: UserId",
                logged_at,
                metric_type,
                value::float8 as "value!",
                notes,
                created_at
            "#,
            raw_id,
            log.user_id as UserId,
            log.logged_at,
            log.metric_type,
            log.value as f64,
            log.notes
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(BiometricLog {
            id: res.id,
            user_id: res.user_id,
            logged_at: res.logged_at,
            metric_type: res.metric_type,
            value: res.value,
            notes: res.notes,
            created_at: res.created_at,
        })
    }

    async fn get_biometric_history(
        &self,
        user_id: UserId,
        page: u64,
        per_page: u64,
    ) -> AppResult<(Vec<BiometricLog>, u64)> {
        let limit = per_page as i64;
        let offset = ((page - 1) * per_page) as i64;

        let total = query_scalar!(
            r#"SELECT count(*) FROM biometric_logs WHERE user_id = $1"#,
            user_id as UserId
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0) as u64;

        let rows = query!(
            r#"
            SELECT
                id as "id: BiometricLogId",
                user_id as "user_id: UserId",
                logged_at,
                metric_type,
                value::float8 as "value!",
                notes,
                created_at
            FROM biometric_logs
            WHERE user_id = $1
            ORDER BY logged_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id as UserId,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let logs = rows
            .into_iter()
            .map(|r| BiometricLog {
                id: r.id,
                user_id: r.user_id,
                logged_at: r.logged_at,
                metric_type: r.metric_type,
                value: r.value,
                notes: r.notes,
                created_at: r.created_at,
            })
            .collect();

        Ok((logs, total))
    }

    async fn get_latest_biometric(
        &self,
        user_id: UserId,
        metric_type: &str,
    ) -> AppResult<Option<BiometricLog>> {
        let row = query!(
            r#"
            SELECT
                id as "id: BiometricLogId",
                user_id as "user_id: UserId",
                logged_at,
                metric_type,
                value::float8 as "value!",
                notes,
                created_at
            FROM biometric_logs
            WHERE user_id = $1 AND metric_type = $2
            ORDER BY logged_at DESC
            LIMIT 1
            "#,
            user_id as UserId,
            metric_type
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| BiometricLog {
            id: r.id,
            user_id: r.user_id,
            logged_at: r.logged_at,
            metric_type: r.metric_type,
            value: r.value,
            notes: r.notes,
            created_at: r.created_at,
        }))
    }

    async fn log_workout(&self, log: WorkoutLog) -> AppResult<WorkoutLog> {
        let raw_id: Uuid = log.id.into();
        let res = query!(
            r#"
            INSERT INTO workout_logs (id, user_id, logged_at, activity_name, duration_minutes, calories_burned, notes)
            VALUES ($1, $2, $3, $4, $5, $6::float8, $7)
            RETURNING
                id as "id: WorkoutLogId",
                user_id as "user_id: UserId",
                logged_at,
                activity_name,
                duration_minutes,
                calories_burned::float8 as "calories_burned!",
                notes,
                created_at
            "#,
            raw_id,
            log.user_id as UserId,
            log.logged_at,
            log.activity_name,
            log.duration_minutes,
            log.calories_burned,
            log.notes
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(WorkoutLog {
            id: res.id,
            user_id: res.user_id,
            logged_at: res.logged_at,
            activity_name: res.activity_name,
            duration_minutes: res.duration_minutes,
            calories_burned: res.calories_burned,
            notes: res.notes,
            created_at: res.created_at,
        })
    }

    async fn get_recent_workouts(
        &self,
        user_id: UserId,
        page: u64,
        per_page: u64,
    ) -> AppResult<(Vec<WorkoutLog>, u64)> {
        let limit = per_page as i64;
        let offset = ((page - 1) * per_page) as i64;

        let total = query_scalar!(
            r#"SELECT count(*) FROM workout_logs WHERE user_id = $1"#,
            user_id as UserId
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0) as u64;

        let rows = query!(
            r#"
            SELECT
                id as "id: WorkoutLogId",
                user_id as "user_id: UserId",
                logged_at,
                activity_name,
                duration_minutes,
                calories_burned::float8 as "calories_burned!",
                notes,
                created_at
            FROM workout_logs
            WHERE user_id = $1
            ORDER BY logged_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id as UserId,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let logs = rows
            .into_iter()
            .map(|r| WorkoutLog {
                id: r.id,
                user_id: r.user_id,
                logged_at: r.logged_at,
                activity_name: r.activity_name,
                duration_minutes: r.duration_minutes,
                calories_burned: r.calories_burned,
                notes: r.notes,
                created_at: r.created_at,
            })
            .collect();

        Ok((logs, total))
    }

    async fn get_active_goals(&self, user_id: UserId) -> AppResult<Vec<UserGoal>> {
        let rows = query!(
            r#"
            SELECT
                id as "id: GoalId",
                user_id as "user_id: UserId",
                category,
                target_type,
                target_value::float8 as "target_value!",
                unit,
                config,
                start_date,
                target_date,
                is_active,
                created_at,
                updated_at
            FROM user_goals
            WHERE user_id = $1 AND is_active = true
            ORDER BY created_at DESC
            "#,
            user_id as UserId
        )
        .fetch_all(&self.pool)
        .await?;

        let goals = rows
            .into_iter()
            .map(|r| UserGoal {
                id: r.id,
                user_id: r.user_id,
                category: r.category,
                target_type: r.target_type,
                target_value: r.target_value,
                unit: r.unit,
                config: r.config,
                start_date: r.start_date,
                target_date: r.target_date,
                is_active: r.is_active,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect();

        Ok(goals)
    }

    async fn create_goal(&self, goal: UserGoal) -> AppResult<UserGoal> {
        let raw_id: Uuid = goal.id.into();
        let res = query!(
            r#"
            INSERT INTO user_goals (id, user_id, category, target_type, target_value, unit, config, start_date, target_date, is_active)
            VALUES ($1, $2, $3, $4, $5::float8, $6, $7, $8, $9, $10)
            RETURNING
                id as "id: GoalId",
                user_id as "user_id: UserId",
                category,
                target_type,
                target_value::float8 as "target_value!",
                unit,
                config,
                start_date,
                target_date,
                is_active,
                created_at,
                updated_at
            "#,
            raw_id,
            goal.user_id as UserId,
            goal.category,
            goal.target_type,
            goal.target_value,
            goal.unit,
            goal.config,
            goal.start_date,
            goal.target_date,
            goal.is_active
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(UserGoal {
            id: res.id,
            user_id: res.user_id,
            category: res.category,
            target_type: res.target_type,
            target_value: res.target_value,
            unit: res.unit,
            config: res.config,
            start_date: res.start_date,
            target_date: res.target_date,
            is_active: res.is_active,
            created_at: res.created_at,
            updated_at: res.updated_at,
        })
    }

    async fn deactivate_goal(&self, user_id: UserId, category: &str) -> AppResult<()> {
        query!(
            r#"
            UPDATE user_goals
            SET is_active = false, updated_at = now()
            WHERE user_id = $1 AND category = $2 AND is_active = true
            "#,
            user_id as UserId,
            category
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
