use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 密钥分组
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Group {
    pub fn new(name: String, parent_id: Option<Uuid>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::now_v7(),
            name,
            parent_id,
            description: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[cfg(test)]
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_new() {
        let group = Group::new("test-group".to_string(), None);
        assert_eq!(group.name, "test-group");
        assert!(group.is_root());
    }

    #[test]
    fn test_group_with_parent() {
        let parent_id = Uuid::now_v7();
        let group = Group::new("child".to_string(), Some(parent_id));
        assert!(!group.is_root());
        assert_eq!(group.parent_id, Some(parent_id));
    }
}