//! Type-safe ID types for the agentic framework.
//!
//! Uses newtype pattern to prevent mixing up different ID types at compile time.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Macro to define a newtype ID wrapper around UUID
macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Create a new random ID
            #[inline]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Create an ID from an existing UUID
            #[inline]
            pub const fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            /// Create an ID from a string, generating a new UUID if parsing fails
            /// This is useful for API responses that may have non-UUID IDs
            #[inline]
            pub fn from_string(s: impl AsRef<str>) -> Self {
                s.as_ref().parse().unwrap_or_else(|_| Self::new())
            }

            /// Get the underlying UUID
            #[inline]
            pub const fn as_uuid(&self) -> &Uuid {
                &self.0
            }

            /// Create a nil (all zeros) ID - useful for testing
            #[inline]
            pub const fn nil() -> Self {
                Self(Uuid::nil())
            }

            /// Check if this is a nil ID
            #[inline]
            pub fn is_nil(&self) -> bool {
                self.0.is_nil()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), &self.0.to_string()[..8])
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl std::str::FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(Uuid::parse_str(s)?))
            }
        }
    };
}

define_id!(
    /// Unique identifier for an agent
    AgentId
);

define_id!(
    /// Unique identifier for a tool
    ToolId
);

define_id!(
    /// Unique identifier for a message
    MessageId
);

define_id!(
    /// Unique identifier for a tool call
    ToolCallId
);

define_id!(
    /// Unique identifier for a memory entry
    MemoryId
);

define_id!(
    /// Unique identifier for a fact in semantic memory
    FactId
);

define_id!(
    /// Unique identifier for a skill in procedural memory
    SkillId
);

define_id!(
    /// Unique identifier for a goal
    GoalId
);

define_id!(
    /// Unique identifier for a plan
    PlanId
);

define_id!(
    /// Unique identifier for an action
    ActionId
);

define_id!(
    /// Unique identifier for a state checkpoint
    CheckpointId
);

define_id!(
    /// Unique identifier for a session
    SessionId
);

define_id!(
    /// Unique identifier for a conversation thread
    ThreadId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_creation() {
        let id1 = AgentId::new();
        let id2 = AgentId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_id_nil() {
        let id = AgentId::nil();
        assert!(id.is_nil());
    }

    #[test]
    fn test_id_from_str() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let id: AgentId = uuid_str.parse().unwrap();
        assert_eq!(id.to_string(), uuid_str);
    }

    #[test]
    fn test_id_debug_format() {
        let id = AgentId::nil();
        let debug = format!("{:?}", id);
        assert!(debug.starts_with("AgentId("));
    }

    #[test]
    fn test_id_serde() {
        let id = AgentId::new();
        let json = serde_json::to_string(&id).unwrap();
        let parsed: AgentId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, parsed);
    }
}
