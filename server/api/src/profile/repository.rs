use crate::common::AppResult;
use crate::common::id::UserId;
use super::models::{Profile, UserPreference, MedicalSafetyProfile, BiometricLog, WorkoutLog, UserGoal};

pub trait ProfileRepository: Send + Sync {
    async fn get_profile(&self, user_id: UserId) -> AppResult<Option<Profile>>;
    async fn upsert_profile(&self, profile: Profile) -> AppResult<Profile>;
    
    async fn get_preferences(&self, user_id: UserId) -> AppResult<Option<UserPreference>>;
    async fn upsert_preferences(&self, prefs: UserPreference) -> AppResult<UserPreference>;
    
    async fn get_medical_safety(&self, user_id: UserId) -> AppResult<Option<MedicalSafetyProfile>>;
    async fn upsert_medical_safety(&self, profile: MedicalSafetyProfile) -> AppResult<MedicalSafetyProfile>;
    
    async fn log_biometric(&self, log: BiometricLog) -> AppResult<BiometricLog>;
    async fn get_biometric_history(&self, user_id: UserId, page: u64, per_page: u64) -> AppResult<(Vec<BiometricLog>, u64)>;
    async fn get_latest_biometric(&self, user_id: UserId, metric_type: &str) -> AppResult<Option<BiometricLog>>;
    
    async fn log_workout(&self, log: WorkoutLog) -> AppResult<WorkoutLog>;
    async fn get_recent_workouts(&self, user_id: UserId, page: u64, per_page: u64) -> AppResult<(Vec<WorkoutLog>, u64)>;
    
    async fn get_active_goals(&self, user_id: UserId) -> AppResult<Vec<UserGoal>>;
    async fn create_goal(&self, goal: UserGoal) -> AppResult<UserGoal>;
    async fn deactivate_goal(&self, user_id: UserId, category: &str) -> AppResult<()>;
}
