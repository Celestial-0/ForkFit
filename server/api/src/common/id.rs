use std::fmt;
use uuid::Uuid;

#[macro_export]
macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self(Uuid::nil())
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<Uuid> for $name {
            fn from(id: Uuid) -> Self {
                Self(id)
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        // sqlx integration for database fields
        impl<'r, DB: sqlx::Database> sqlx::Decode<'r, DB> for $name
        where
            Uuid: sqlx::Decode<'r, DB>,
        {
            fn decode(
                value: DB::ValueRef<'r>,
            ) -> Result<Self, sqlx::error::BoxDynError> {
                let uuid = <Uuid as sqlx::Decode<'r, DB>>::decode(value)?;
                Ok(Self(uuid))
            }
        }

        impl<DB: sqlx::Database> sqlx::Type<DB> for $name
        where
            Uuid: sqlx::Type<DB>,
        {
            fn type_info() -> DB::TypeInfo {
                <Uuid as sqlx::Type<DB>>::type_info()
            }
        }

        impl<'q, DB: sqlx::Database> sqlx::Encode<'q, DB> for $name
        where
            Uuid: sqlx::Encode<'q, DB>,
        {
            fn encode_by_ref(
                &self,
                buf: &mut DB::ArgumentBuffer,
            ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
                <Uuid as sqlx::Encode<'q, DB>>::encode_by_ref(&self.0, buf)
            }
        }
    };
}

define_id!(UserId);
define_id!(RecipeId);
define_id!(FoodItemId);
define_id!(MealPlanId);
define_id!(ChatThreadId);
define_id!(ChatMessageId);
define_id!(TraceId);
define_id!(SessionId);
define_id!(FeedbackId);
define_id!(MemoryId);
define_id!(ShoppingListId);
define_id!(BiometricLogId);
define_id!(WorkoutLogId);
define_id!(GoalId);
define_id!(FoodLogId);
define_id!(MealPlanItemId);
define_id!(PantryItemId);
define_id!(ShoppingListItemId);
define_id!(BackgroundJobId);
define_id!(NotificationLogId);
define_id!(AuditLogId);
define_id!(FoodItemPortionId);
define_id!(RawFoodCostId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newtype_id_display() {
        let uuid = Uuid::parse_str("67e55044-10b1-426f-9247-bb680e5fe0c8").unwrap();
        let user_id = UserId(uuid);
        assert_eq!(format!("{user_id}"), "67e55044-10b1-426f-9247-bb680e5fe0c8");
    }

    #[test]
    fn test_newtype_id_default() {
        let user_id = UserId::default();
        assert_eq!(user_id.0, Uuid::nil());
    }

    #[test]
    fn test_newtype_id_serde_round_trip() {
        let uuid = Uuid::new_v4();
        let user_id = UserId(uuid);
        let serialized = serde_json::to_string(&user_id).unwrap();
        let deserialized: UserId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, user_id);
    }
}

