//! DM Memory for context management.
//!
//! Manages conversation history and context for long-running campaigns.
//! Implements a hybrid approach: shared campaign facts + sliding window
//! of recent conversation.

use claude::Message;
use serde::{Deserialize, Serialize};

/// Maximum number of recent messages to keep in full detail.
const MAX_RECENT_MESSAGES: usize = 20;

/// DM Memory manages context for the AI Dungeon Master.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmMemory {
    /// Key campaign facts that persist indefinitely.
    pub campaign_facts: Vec<CampaignFact>,

    /// Recent conversation history (sliding window).
    recent_messages: Vec<StoredMessage>,

    /// Summary of older conversations.
    pub conversation_summary: Option<String>,

    /// Token budget for context management.
    pub token_budget: usize,
}

impl DmMemory {
    /// Create a new DM memory with default settings.
    pub fn new() -> Self {
        Self {
            campaign_facts: Vec::new(),
            recent_messages: Vec::new(),
            conversation_summary: None,
            token_budget: 100_000, // Default 100k token budget
        }
    }

    /// Create with a specific token budget.
    pub fn with_budget(token_budget: usize) -> Self {
        Self {
            token_budget,
            ..Self::new()
        }
    }

    /// Add a player message to history.
    pub fn add_player_message(&mut self, content: &str) {
        self.recent_messages.push(StoredMessage {
            role: MessageRole::User,
            content: content.to_string(),
        });
        self.trim_history();
    }

    /// Add a DM response to history.
    pub fn add_dm_message(&mut self, content: &str) {
        self.recent_messages.push(StoredMessage {
            role: MessageRole::Assistant,
            content: content.to_string(),
        });
        self.trim_history();
    }

    /// Add a campaign fact (persists indefinitely).
    pub fn add_fact(&mut self, category: FactCategory, content: impl Into<String>) {
        self.campaign_facts.push(CampaignFact {
            category,
            content: content.into(),
        });
    }

    /// Get messages for API call.
    pub fn get_messages(&self) -> Vec<Message> {
        self.recent_messages
            .iter()
            .map(|m| match m.role {
                MessageRole::User => Message::user(&m.content),
                MessageRole::Assistant => Message::assistant(&m.content),
            })
            .collect()
    }

    /// Build context string with campaign facts.
    pub fn build_context(&self) -> String {
        let mut context = String::new();

        // Add conversation summary if available
        if let Some(ref summary) = self.conversation_summary {
            context.push_str("## Previous Session Summary\n");
            context.push_str(summary);
            context.push_str("\n\n");
        }

        // Add campaign facts by category
        if !self.campaign_facts.is_empty() {
            context.push_str("## Campaign Facts\n");

            // Group by category
            let mut npcs = Vec::new();
            let mut locations = Vec::new();
            let mut quests = Vec::new();
            let mut lore = Vec::new();
            let mut other = Vec::new();

            for fact in &self.campaign_facts {
                match fact.category {
                    FactCategory::NPC => npcs.push(&fact.content),
                    FactCategory::Location => locations.push(&fact.content),
                    FactCategory::Quest => quests.push(&fact.content),
                    FactCategory::Lore => lore.push(&fact.content),
                    FactCategory::Other => other.push(&fact.content),
                }
            }

            if !npcs.is_empty() {
                context.push_str("\n### NPCs\n");
                for npc in npcs {
                    context.push_str(&format!("- {npc}\n"));
                }
            }

            if !locations.is_empty() {
                context.push_str("\n### Locations\n");
                for loc in locations {
                    context.push_str(&format!("- {loc}\n"));
                }
            }

            if !quests.is_empty() {
                context.push_str("\n### Quests\n");
                for quest in quests {
                    context.push_str(&format!("- {quest}\n"));
                }
            }

            if !lore.is_empty() {
                context.push_str("\n### Lore\n");
                for lore_item in lore {
                    context.push_str(&format!("- {lore_item}\n"));
                }
            }

            if !other.is_empty() {
                context.push_str("\n### Other\n");
                for item in other {
                    context.push_str(&format!("- {item}\n"));
                }
            }
        }

        context
    }

    /// Set conversation summary (from previous session).
    pub fn set_summary(&mut self, summary: impl Into<String>) {
        self.conversation_summary = Some(summary.into());
    }

    /// Generate a summary of the current conversation for persistence.
    pub fn generate_summary(&self) -> String {
        // This is a simplified summary - in production, you might use
        // an LLM to generate a proper summary
        let mut summary = String::new();

        let message_count = self.recent_messages.len();
        summary.push_str(&format!("Session with {} exchanges.\n", message_count / 2));

        // Include last few player actions
        let player_actions: Vec<_> = self.recent_messages
            .iter()
            .filter(|m| matches!(m.role, MessageRole::User))
            .map(|m| &m.content)
            .rev()
            .take(5)
            .collect();

        if !player_actions.is_empty() {
            summary.push_str("Recent player actions:\n");
            for action in player_actions.iter().rev() {
                // Truncate long messages
                let truncated = if action.len() > 100 {
                    format!("{}...", &action[..100])
                } else {
                    action.to_string()
                };
                summary.push_str(&format!("- {truncated}\n"));
            }
        }

        summary
    }

    /// Clear conversation history but keep campaign facts.
    pub fn clear_conversation(&mut self) {
        self.recent_messages.clear();
    }

    /// Get the number of stored messages.
    pub fn message_count(&self) -> usize {
        self.recent_messages.len()
    }

    fn trim_history(&mut self) {
        while self.recent_messages.len() > MAX_RECENT_MESSAGES {
            self.recent_messages.remove(0);
        }
    }
}

impl Default for DmMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// A stored message in the conversation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredMessage {
    role: MessageRole,
    content: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum MessageRole {
    User,
    Assistant,
}

/// A persisted campaign fact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignFact {
    pub category: FactCategory,
    pub content: String,
}

/// Categories for campaign facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FactCategory {
    NPC,
    Location,
    Quest,
    Lore,
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let memory = DmMemory::new();
        assert_eq!(memory.message_count(), 0);
        assert!(memory.campaign_facts.is_empty());
    }

    #[test]
    fn test_add_messages() {
        let mut memory = DmMemory::new();
        memory.add_player_message("I attack the goblin");
        memory.add_dm_message("Roll for attack!");

        assert_eq!(memory.message_count(), 2);
    }

    #[test]
    fn test_add_facts() {
        let mut memory = DmMemory::new();
        memory.add_fact(FactCategory::NPC, "Gandalf is a wizard");
        memory.add_fact(FactCategory::Location, "Moria is dangerous");

        assert_eq!(memory.campaign_facts.len(), 2);
    }

    #[test]
    fn test_trim_history() {
        let mut memory = DmMemory::new();

        for i in 0..30 {
            memory.add_player_message(&format!("Message {i}"));
        }

        assert_eq!(memory.message_count(), MAX_RECENT_MESSAGES);
    }

    #[test]
    fn test_get_messages() {
        let mut memory = DmMemory::new();
        memory.add_player_message("Hello");
        memory.add_dm_message("Greetings, adventurer!");

        let messages = memory.get_messages();
        assert_eq!(messages.len(), 2);
    }
}
