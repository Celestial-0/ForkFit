use chrono::Utc;

use crate::common::AppResult;
use crate::common::id::{UserId, BiometricLogId, WorkoutLogId, GoalId};
use crate::common::error::AppError;

use super::models::{Profile, UserPreference, MedicalSafetyProfile, BiometricLog, WorkoutLog, UserGoal};
use super::repository::ProfileRepository;
use super::error::ProfileError;
use super::types::{
    UpdateProfileRequest, UpdateMedicalSafetyRequest, UpdatePreferenceRequest,
    CreateBiometricRequest, CreateWorkoutRequest, CreateGoalRequest,
};

#[derive(Clone)]
pub struct ProfileService<R> {
    repo: R,
}

impl<R: ProfileRepository> ProfileService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn get_profile(&self, user_id: UserId) -> AppResult<Profile> {
        let profile = self.repo.get_profile(user_id).await?;
        match profile {
            Some(p) => Ok(p),
            None => Err(ProfileError::NotFound(user_id).into()),
        }
    }

    pub async fn update_profile(&self, user_id: UserId, req: UpdateProfileRequest) -> AppResult<Profile> {
        if let Some(dob) = req.dob {
            if dob > Utc::now().date_naive() {
                return Err(ProfileError::InvalidDateOfBirth.into());
            }
        }

        let profile = Profile {
            user_id,
            full_name: req.full_name,
            avatar_url: req.avatar_url,
            gender: req.gender,
            dob: req.dob,
            timezone: req.timezone,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.repo.upsert_profile(profile).await
    }

    pub async fn get_preferences(&self, user_id: UserId) -> AppResult<UserPreference> {
        let prefs = self.repo.get_preferences(user_id).await?;
        match prefs {
            Some(p) => Ok(p),
            None => {
                // Return defaults if not configured
                Ok(UserPreference {
                    user_id,
                    theme: "light".to_string(),
                    language: "en".to_string(),
                    measurement_system: "metric".to_string(),
                    preferences: serde_json::json!({}),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                })
            }
        }
    }

    pub async fn update_preferences(&self, user_id: UserId, req: UpdatePreferenceRequest) -> AppResult<UserPreference> {
        let prefs = UserPreference {
            user_id,
            theme: req.theme,
            language: req.language,
            measurement_system: req.measurement_system,
            preferences: req.preferences,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.repo.upsert_preferences(prefs).await
    }

    pub async fn get_medical_safety(&self, user_id: UserId) -> AppResult<MedicalSafetyProfile> {
        let safety = self.repo.get_medical_safety(user_id).await?;
        match safety {
            Some(s) => Ok(s),
            None => {
                // Return default empty safety profile
                Ok(MedicalSafetyProfile {
                    user_id,
                    allergies: vec![],
                    medical_conditions: vec![],
                    is_pregnant: false,
                    is_lactating: false,
                    updated_at: Utc::now(),
                })
            }
        }
    }

    pub async fn update_medical_safety(&self, user_id: UserId, req: UpdateMedicalSafetyRequest) -> AppResult<MedicalSafetyProfile> {
        // Allergy name validation
        for allergy in &req.allergies {
            let trimmed = allergy.trim();
            if trimmed.is_empty() {
                return Err(ProfileError::ValidationError("Allergy name cannot be empty".to_string()).into());
            }
            if trimmed.len() > 100 {
                return Err(ProfileError::ValidationError("Allergy name cannot exceed 100 characters".to_string()).into());
            }
        }

        // Medical conditions validation
        for condition in &req.medical_conditions {
            let trimmed = condition.trim();
            if trimmed.is_empty() {
                return Err(ProfileError::ValidationError("Medical condition cannot be empty".to_string()).into());
            }
            if trimmed.len() > 100 {
                return Err(ProfileError::ValidationError("Medical condition cannot exceed 100 characters".to_string()).into());
            }
        }

        let safety = MedicalSafetyProfile {
            user_id,
            allergies: req.allergies.into_iter().map(|s| s.trim().to_string()).collect(),
            medical_conditions: req.medical_conditions.into_iter().map(|s| s.trim().to_string()).collect(),
            is_pregnant: req.is_pregnant,
            is_lactating: req.is_lactating,
            updated_at: Utc::now(),
        };

        self.repo.upsert_medical_safety(safety).await
    }

    pub async fn log_biometric(&self, user_id: UserId, req: CreateBiometricRequest) -> AppResult<BiometricLog> {
        let metric_type_lower = req.metric_type.to_lowercase();
        
        // Validation: weight bounds
        if metric_type_lower == "weight" {
            if req.value < 10.0 || req.value > 500.0 {
                return Err(ProfileError::InvalidBiometric(
                    "Weight must be between 10kg and 500kg".to_string()
                ).into());
            }
        }

        // Validation: height bounds
        if metric_type_lower == "height" {
            if req.value < 50.0 || req.value > 300.0 {
                return Err(ProfileError::InvalidBiometric(
                    "Height must be between 50cm and 300cm".to_string()
                ).into());
            }
        }

        // Validation: generic value bounds
        if req.value <= 0.0 {
            return Err(ProfileError::InvalidBiometric(
                "Biometric value must be positive".to_string()
            ).into());
        }

        let log = BiometricLog {
            id: BiometricLogId::new(),
            user_id,
            logged_at: req.logged_at.unwrap_or_else(Utc::now),
            metric_type: req.metric_type,
            value: req.value,
            notes: req.notes,
            created_at: Utc::now(),
        };

        self.repo.log_biometric(log).await
    }

    pub async fn get_biometric_history(&self, user_id: UserId, page: u64, per_page: u64) -> AppResult<(Vec<BiometricLog>, u64)> {
        self.repo.get_biometric_history(user_id, page, per_page).await
    }

    pub async fn log_workout(&self, user_id: UserId, req: CreateWorkoutRequest) -> AppResult<WorkoutLog> {
        if req.duration_minutes <= 0 {
            return Err(AppError::BadRequest("Workout duration must be positive".to_string()));
        }
        if req.calories_burned < 0.0 {
            return Err(AppError::BadRequest("Calories burned cannot be negative".to_string()));
        }

        let log = WorkoutLog {
            id: WorkoutLogId::new(),
            user_id,
            logged_at: req.logged_at.unwrap_or_else(Utc::now),
            activity_name: req.activity_name,
            duration_minutes: req.duration_minutes,
            calories_burned: req.calories_burned,
            notes: req.notes,
            created_at: Utc::now(),
        };

        self.repo.log_workout(log).await
    }

    pub async fn get_recent_workouts(&self, user_id: UserId, page: u64, per_page: u64) -> AppResult<(Vec<WorkoutLog>, u64)> {
        self.repo.get_recent_workouts(user_id, page, per_page).await
    }

    pub async fn get_active_goals(&self, user_id: UserId) -> AppResult<Vec<UserGoal>> {
        self.repo.get_active_goals(user_id).await
    }

    pub async fn create_goal(&self, user_id: UserId, req: CreateGoalRequest) -> AppResult<UserGoal> {
        if req.category.trim().is_empty() {
            return Err(AppError::BadRequest("Goal category cannot be empty".to_string()));
        }
        if req.target_type.trim().is_empty() {
            return Err(AppError::BadRequest("Goal target type cannot be empty".to_string()));
        }
        if req.target_value <= 0.0 {
            return Err(AppError::BadRequest("Goal target value must be positive".to_string()));
        }

        // Deactivate existing goals in this category
        self.repo.deactivate_goal(user_id, &req.category).await?;

        let goal = UserGoal {
            id: GoalId::new(),
            user_id,
            category: req.category,
            target_type: req.target_type,
            target_value: req.target_value,
            unit: req.unit,
            config: req.config,
            start_date: req.start_date.unwrap_or_else(|| Utc::now().date_naive()),
            target_date: req.target_date,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.repo.create_goal(goal).await
    }

    pub async fn deactivate_goal(&self, user_id: UserId, category: &str) -> AppResult<()> {
        self.repo.deactivate_goal(user_id, category).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use crate::common::id::UserId;
    use uuid::Uuid;

    #[derive(Default)]
    struct MockProfileRepository {
        deactivated_categories: Mutex<Vec<String>>,
    }

    impl ProfileRepository for MockProfileRepository {
        async fn get_profile(&self, _user_id: UserId) -> AppResult<Option<Profile>> {
            Ok(None)
        }

        async fn upsert_profile(&self, profile: Profile) -> AppResult<Profile> {
            Ok(profile)
        }

        async fn get_preferences(&self, _user_id: UserId) -> AppResult<Option<UserPreference>> {
            Ok(None)
        }

        async fn upsert_preferences(&self, prefs: UserPreference) -> AppResult<UserPreference> {
            Ok(prefs)
        }

        async fn get_medical_safety(&self, _user_id: UserId) -> AppResult<Option<MedicalSafetyProfile>> {
            Ok(None)
        }

        async fn upsert_medical_safety(&self, profile: MedicalSafetyProfile) -> AppResult<MedicalSafetyProfile> {
            Ok(profile)
        }

        async fn log_biometric(&self, log: BiometricLog) -> AppResult<BiometricLog> {
            Ok(log)
        }

        async fn get_biometric_history(&self, _user_id: UserId, _page: u64, _per_page: u64) -> AppResult<(Vec<BiometricLog>, u64)> {
            Ok((vec![], 0))
        }

        async fn get_latest_biometric(&self, _user_id: UserId, _metric_type: &str) -> AppResult<Option<BiometricLog>> {
            Ok(None)
        }

        async fn log_workout(&self, log: WorkoutLog) -> AppResult<WorkoutLog> {
            Ok(log)
        }

        async fn get_recent_workouts(&self, _user_id: UserId, _page: u64, _per_page: u64) -> AppResult<(Vec<WorkoutLog>, u64)> {
            Ok((vec![], 0))
        }

        async fn get_active_goals(&self, _user_id: UserId) -> AppResult<Vec<UserGoal>> {
            Ok(vec![])
        }

        async fn create_goal(&self, goal: UserGoal) -> AppResult<UserGoal> {
            Ok(goal)
        }

        async fn deactivate_goal(&self, _user_id: UserId, category: &str) -> AppResult<()> {
            self.deactivated_categories.lock().unwrap().push(category.to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_log_biometric_weight_bounds() {
        let repo = MockProfileRepository::default();
        let service = ProfileService::new(repo);
        let user_id = UserId(Uuid::new_v4());

        // Valid weight
        let res = service.log_biometric(user_id, CreateBiometricRequest {
            metric_type: "Weight".to_string(),
            value: 70.0,
            notes: None,
            logged_at: None,
        }).await;
        assert!(res.is_ok());

        // Invalid weight (too low)
        let res = service.log_biometric(user_id, CreateBiometricRequest {
            metric_type: "Weight".to_string(),
            value: 5.0,
            notes: None,
            logged_at: None,
        }).await;
        assert!(res.is_err());

        // Invalid weight (too high)
        let res = service.log_biometric(user_id, CreateBiometricRequest {
            metric_type: "Weight".to_string(),
            value: 505.0,
            notes: None,
            logged_at: None,
        }).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_log_biometric_height_bounds() {
        let repo = MockProfileRepository::default();
        let service = ProfileService::new(repo);
        let user_id = UserId(Uuid::new_v4());

        // Valid height
        let res = service.log_biometric(user_id, CreateBiometricRequest {
            metric_type: "Height".to_string(),
            value: 175.0,
            notes: None,
            logged_at: None,
        }).await;
        assert!(res.is_ok());

        // Invalid height (too low)
        let res = service.log_biometric(user_id, CreateBiometricRequest {
            metric_type: "Height".to_string(),
            value: 45.0,
            notes: None,
            logged_at: None,
        }).await;
        assert!(res.is_err());

        // Invalid height (too high)
        let res = service.log_biometric(user_id, CreateBiometricRequest {
            metric_type: "Height".to_string(),
            value: 305.0,
            notes: None,
            logged_at: None,
        }).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_goal_deactivation_on_create() {
        let repo = MockProfileRepository::default();
        let service = ProfileService::new(repo);
        let user_id = UserId(Uuid::new_v4());

        let res = service.create_goal(user_id, CreateGoalRequest {
            category: "nutrition".to_string(),
            target_type: "calorie_surplus".to_string(),
            target_value: 3000.0,
            unit: "kcal".to_string(),
            config: serde_json::json!({}),
            start_date: None,
            target_date: None,
        }).await;
        assert!(res.is_ok());

        // Assert that the deactivate_goal repo method was called with category "nutrition"
        let deactivated = service.repo.deactivated_categories.lock().unwrap();
        assert_eq!(deactivated.len(), 1);
        assert_eq!(deactivated[0], "nutrition");
    }

    #[tokio::test]
    async fn test_update_medical_safety_allergy_validation() {
        let repo = MockProfileRepository::default();
        let service = ProfileService::new(repo);
        let user_id = UserId(Uuid::new_v4());

        // Valid allergy list
        let res = service.update_medical_safety(user_id, UpdateMedicalSafetyRequest {
            allergies: vec!["Peanuts".to_string(), "Shellfish".to_string()],
            medical_conditions: vec![],
            is_pregnant: false,
            is_lactating: false,
        }).await;
        assert!(res.is_ok());

        // Invalid allergy list (empty string)
        let res = service.update_medical_safety(user_id, UpdateMedicalSafetyRequest {
            allergies: vec!["".to_string()],
            medical_conditions: vec![],
            is_pregnant: false,
            is_lactating: false,
        }).await;
        assert!(res.is_err());

        // Invalid allergy list (too long string)
        let long_allergy = "a".repeat(101);
        let res = service.update_medical_safety(user_id, UpdateMedicalSafetyRequest {
            allergies: vec![long_allergy],
            medical_conditions: vec![],
            is_pregnant: false,
            is_lactating: false,
        }).await;
        assert!(res.is_err());
    }
}

