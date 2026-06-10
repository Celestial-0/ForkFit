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
define_id!(IngredientId);
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
