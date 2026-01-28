//! AI Dungeon Master agent.
//!
//! The DungeonMaster struct provides the main interface for AI-powered
//! D&D gameplay. It uses the Claude API to generate narrative responses
//! and tool calls that are resolved by the RulesEngine.

use super::memory::{DmMemory, FactCategory};
use super::story_memory::{EntityType, FactCategory as StoryFactCategory, FactSource, StoryMemory};
use super::tools::{DmTools, parse_tool_call};
use crate::rules::{apply_effects, Effect, Intent, Resolution, RulesEngine};
use crate::world::{GameMode, GameWorld, NarrativeType};
use claude::{Claude, ContentBlock, Message, Request, StopReason, ToolResult};
use thiserror::Error;

/// Errors from the DM agent.
#[derive(Debug, Error)]
pub enum DmError {
    #[error("Claude API error: {0:?}")]
    ApiError(#[from] claude::Error),

    #[error("No API key configured")]
    NoApiKey,

    #[error("Tool execution failed: {0}")]
    ToolError(String),
}

/// Configuration for the Dungeon Master.
#[derive(Debug, Clone)]
pub struct DmConfig {
    /// The model to use (defaults to claude-sonnet-4-20250514).
    pub model: Option<String>,

    /// Maximum tokens for responses.
    pub max_tokens: usize,

    /// Temperature for generation.
    pub temperature: Option<f32>,

    /// System prompt customization.
    pub custom_system_prompt: Option<String>,
}

impl Default for DmConfig {
    fn default() -> Self {
        Self {
            model: None,
            max_tokens: 4096,
            temperature: Some(0.8),
            custom_system_prompt: None,
        }
    }
}

/// Response from the Dungeon Master.
#[derive(Debug)]
pub struct DmResponse {
    /// The narrative text from the DM.
    pub narrative: String,

    /// Intents generated from tool calls.
    pub intents: Vec<Intent>,

    /// Effects from resolving intents.
    pub effects: Vec<Effect>,

    /// Resolution details for each intent.
    pub resolutions: Vec<Resolution>,
}

/// The AI Dungeon Master.
pub struct DungeonMaster {
    client: Claude,
    config: DmConfig,
    memory: DmMemory,
    story_memory: StoryMemory,
    rules: RulesEngine,
}

impl DungeonMaster {
    /// Create a new DungeonMaster with an API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Claude::new(api_key),
            config: DmConfig::default(),
            memory: DmMemory::new(),
            story_memory: StoryMemory::new(),
            rules: RulesEngine::new(),
        }
    }

    /// Create a DungeonMaster from the ANTHROPIC_API_KEY environment variable.
    pub fn from_env() -> Result<Self, DmError> {
        let client = Claude::from_env()?;
        Ok(Self {
            client,
            config: DmConfig::default(),
            memory: DmMemory::new(),
            story_memory: StoryMemory::new(),
            rules: RulesEngine::new(),
        })
    }

    /// Get the story memory.
    pub fn story_memory(&self) -> &StoryMemory {
        &self.story_memory
    }

    /// Get mutable access to story memory.
    pub fn story_memory_mut(&mut self) -> &mut StoryMemory {
        &mut self.story_memory
    }

    /// Configure the DungeonMaster.
    pub fn with_config(mut self, config: DmConfig) -> Self {
        self.config = config;
        self
    }

    /// Get the current memory.
    pub fn memory(&self) -> &DmMemory {
        &self.memory
    }

    /// Get mutable access to memory.
    pub fn memory_mut(&mut self) -> &mut DmMemory {
        &mut self.memory
    }

    /// Process a player's action and generate a response.
    pub async fn process_input(
        &mut self,
        player_input: &str,
        world: &mut GameWorld,
    ) -> Result<DmResponse, DmError> {
        // Advance story turn
        self.story_memory.advance_turn();

        // Add player input to memory
        self.memory.add_player_message(player_input);

        // Add to game world narrative
        world.add_narrative(player_input.to_string(), NarrativeType::PlayerAction);

        // Build system prompt with story context for this input
        let system_prompt = self.build_system_prompt(world, player_input);

        // Track intents, effects, and resolutions
        let mut all_intents = Vec::new();
        let mut all_effects = Vec::new();
        let mut all_resolutions = Vec::new();
        let mut narrative = String::new();

        // Build initial messages
        let mut messages = self.memory.get_messages();

        // Tool use loop
        loop {
            let tools = DmTools::all();

            let mut request = Request::new(messages.clone())
                .with_system(&system_prompt)
                .with_max_tokens(self.config.max_tokens)
                .with_tools(tools);

            if let Some(ref model) = self.config.model {
                request = request.with_model(model);
            }

            if let Some(temp) = self.config.temperature {
                request = request.with_temperature(temp);
            }

            // Make API call
            let response = self.client.complete(request).await?;

            // Collect tool uses
            let mut tool_uses = Vec::new();
            for block in &response.content {
                match block {
                    ContentBlock::Text { text } => {
                        if !narrative.is_empty() {
                            narrative.push('\n');
                        }
                        narrative.push_str(text);
                    }
                    ContentBlock::ToolUse { id, name, input } => {
                        tool_uses.push((id.clone(), name.clone(), input.clone()));
                    }
                    _ => {}
                }
            }

            // If no tool calls or stop reason isn't ToolUse, we're done
            if response.stop_reason != StopReason::ToolUse || tool_uses.is_empty() {
                break;
            }

            // Add assistant response to messages
            messages.push(Message {
                role: claude::Role::Assistant,
                content: response.content.clone(),
            });

            // Execute tools and collect results
            let mut tool_results = Vec::new();
            for (id, name, input) in tool_uses {
                let result = if let Some(intent) = parse_tool_call(&name, &input, world) {
                    // Resolve the intent
                    let resolution = self.rules.resolve(world, intent.clone());

                    // Apply effects to world
                    apply_effects(world, &resolution.effects);

                    // Handle FactRemembered effects specially - store in story memory
                    for effect in &resolution.effects {
                        if let Effect::FactRemembered {
                            subject_name,
                            subject_type,
                            fact,
                            category,
                            related_entities,
                            importance,
                        } = effect
                        {
                            self.store_fact(
                                subject_name,
                                subject_type,
                                fact,
                                category,
                                related_entities,
                                *importance,
                            );
                        }
                    }

                    // Store for response
                    all_intents.push(intent);
                    all_effects.extend(resolution.effects.clone());
                    all_resolutions.push(resolution.clone());

                    // Return narrative as tool result
                    ToolResult::success(&resolution.narrative)
                } else {
                    ToolResult::error(format!("Unknown tool: {name}"))
                };

                tool_results.push(ContentBlock::ToolResult {
                    tool_use_id: id,
                    content: result.content,
                    is_error: result.is_error,
                });
            }

            // Add tool results as user message
            messages.push(Message {
                role: claude::Role::User,
                content: tool_results,
            });
        }

        // Add DM response to memory
        self.memory.add_dm_message(&narrative);

        // Add to game world narrative
        world.add_narrative(narrative.clone(), NarrativeType::DmNarration);

        Ok(DmResponse {
            narrative,
            intents: all_intents,
            effects: all_effects,
            resolutions: all_resolutions,
        })
    }

    fn build_system_prompt(&self, world: &GameWorld, player_input: &str) -> String {
        let mut prompt = String::new();

        // Base DM prompt
        prompt.push_str(include_str!("prompts/dm_base.txt"));

        // Add story memory instructions
        prompt.push_str("\n\n");
        prompt.push_str(include_str!("prompts/story_memory.txt"));

        // Add background-based adventure hooks
        prompt.push_str("\n\n");
        prompt.push_str(include_str!("prompts/background_hooks.txt"));

        // Add combat triggers - when to initiate combat
        prompt.push_str("\n\n");
        prompt.push_str(include_str!("prompts/combat_triggers.txt"));

        // Add combat turn management - how to track rounds and turns
        prompt.push_str("\n\n");
        prompt.push_str(include_str!("prompts/combat_turns.txt"));

        // Add encounter pacing for solo adventures
        prompt.push_str("\n\n");
        prompt.push_str(include_str!("prompts/encounter_pacing.txt"));

        // Add rest rules to prevent hallucinated restrictions
        prompt.push_str("\n\n");
        prompt.push_str(include_str!("prompts/rest_rules.txt"));

        // Add skill check requirements - CRITICAL for dice rolling
        prompt.push_str("\n\n");
        prompt.push_str(include_str!("prompts/skill_checks.txt"));

        // Add class feature awareness - prompt DM to offer class abilities
        prompt.push_str("\n\n");
        prompt.push_str(include_str!("prompts/class_features.txt"));

        // Add custom prompt if provided
        if let Some(ref custom) = self.config.custom_system_prompt {
            prompt.push_str("\n\n## Additional Instructions\n");
            prompt.push_str(custom);
        }

        // Add campaign context
        prompt.push_str("\n\n## Current Campaign: ");
        prompt.push_str(&world.campaign_name);
        prompt.push('\n');

        // Add player character info
        prompt.push_str("\n## Player Character\n");
        let pc = &world.player_character;
        prompt.push_str(&format!("**Name:** {}\n", pc.name));
        prompt.push_str(&format!("**Level:** {}", pc.level));
        if !pc.classes.is_empty() {
            let class_info: Vec<_> = pc.classes.iter()
                .map(|c| format!("{} {}", c.class.name(), c.level))
                .collect();
            prompt.push_str(&format!(" ({})", class_info.join("/")));
        }
        prompt.push('\n');
        prompt.push_str(&format!("**Race:** {}\n", pc.race.name));
        prompt.push_str(&format!("**Background:** {} - {}\n", pc.background.name(), pc.background.description()));
        prompt.push_str(&format!("**HP:** {}/{}\n", pc.hit_points.current, pc.hit_points.maximum));
        prompt.push_str(&format!("**AC:** {}\n", pc.current_ac()));

        // Add ability scores
        prompt.push_str(&format!(
            "**Abilities:** STR {} DEX {} CON {} INT {} WIS {} CHA {}\n",
            pc.ability_scores.strength,
            pc.ability_scores.dexterity,
            pc.ability_scores.constitution,
            pc.ability_scores.intelligence,
            pc.ability_scores.wisdom,
            pc.ability_scores.charisma
        ));

        // Add current situation
        prompt.push_str("\n## Current Situation\n");
        prompt.push_str(&format!("Location: {}\n", world.current_location.name));
        prompt.push_str(&format!("Time: {} ({})\n",
            world.game_time.time_of_day(),
            if world.game_time.is_daytime() { "day" } else { "night" }
        ));
        prompt.push_str(&format!("Mode: {:?}\n", world.mode));

        // Combat info if in combat
        if world.mode == GameMode::Combat {
            // Include combat-specific narration guidelines
            prompt.push('\n');
            prompt.push_str(include_str!("prompts/combat.txt"));

            if let Some(ref combat) = world.combat {
                prompt.push_str(&format!("\n### Combat Status - Round {}\n", combat.round));
                if let Some(current) = combat.current_combatant() {
                    prompt.push_str(&format!("**Current turn:** {}\n", current.name));
                }
                prompt.push_str("\n**Initiative Order:**\n");
                for (i, c) in combat.combatants.iter().enumerate() {
                    let marker = if i == combat.turn_index { ">" } else { " " };
                    let hp_status = Self::describe_hp_status(c.current_hp, c.max_hp);
                    prompt.push_str(&format!(
                        "{} {}. {} (init {}) - {}\n",
                        marker,
                        i + 1,
                        c.name,
                        c.initiative,
                        hp_status
                    ));
                }
            }
        }

        // Active conditions
        if !pc.conditions.is_empty() {
            prompt.push_str("\nActive conditions:\n");
            for cond in &pc.conditions {
                prompt.push_str(&format!("- {} (from {})\n", cond.condition, cond.source));
            }
        }

        // Add memory context
        let memory_context = self.memory.build_context();
        if !memory_context.is_empty() {
            prompt.push('\n');
            prompt.push_str(&memory_context);
        }

        // Add story memory context for entities mentioned in player input
        let story_context = self.story_memory.build_context_for_input(player_input);
        if !story_context.is_empty() {
            prompt.push('\n');
            prompt.push_str(&story_context);
        }

        prompt
    }

    /// Add a campaign fact to memory.
    pub fn remember(&mut self, category: FactCategory, fact: impl Into<String>) {
        self.memory.add_fact(category, fact);
    }

    /// Describe HP status in narrative terms (for combat display).
    fn describe_hp_status(current: i32, max: i32) -> &'static str {
        if current <= 0 {
            "down"
        } else if current == max {
            "uninjured"
        } else {
            let ratio = current as f32 / max as f32;
            if ratio > 0.75 {
                "lightly wounded"
            } else if ratio > 0.5 {
                "bloodied"
            } else if ratio > 0.25 {
                "badly wounded"
            } else {
                "near death"
            }
        }
    }

    /// Store a fact in story memory.
    fn store_fact(
        &mut self,
        subject_name: &str,
        subject_type: &str,
        fact: &str,
        category: &str,
        related_entities: &[String],
        importance: f32,
    ) {
        // Parse entity type
        let entity_type = match subject_type.to_lowercase().as_str() {
            "npc" => EntityType::Npc,
            "location" => EntityType::Location,
            "item" => EntityType::Item,
            "quest" => EntityType::Quest,
            "organization" => EntityType::Organization,
            "event" => EntityType::Event,
            "creature" => EntityType::Creature,
            _ => EntityType::Npc, // Default to NPC
        };

        // Parse fact category
        let fact_category = match category.to_lowercase().as_str() {
            "appearance" => StoryFactCategory::Appearance,
            "personality" => StoryFactCategory::Personality,
            "event" => StoryFactCategory::Event,
            "relationship" => StoryFactCategory::Relationship,
            "backstory" => StoryFactCategory::Backstory,
            "motivation" => StoryFactCategory::Motivation,
            "capability" => StoryFactCategory::Capability,
            "location" => StoryFactCategory::Location,
            "possession" => StoryFactCategory::Possession,
            "status" => StoryFactCategory::Status,
            "secret" => StoryFactCategory::Secret,
            _ => StoryFactCategory::Event, // Default
        };

        // Get or create the subject entity
        let subject_id = self
            .story_memory
            .get_or_create_entity(entity_type, subject_name);

        // Resolve related entity IDs
        let mut mentioned_ids = Vec::new();
        for name in related_entities {
            // Try to find existing entity, or skip if not found
            // (we don't auto-create related entities)
            if let Some(id) = self.story_memory.find_entity_id(name) {
                mentioned_ids.push(id);
            }
        }

        // Record the fact with the specified importance
        self.story_memory.record_fact_full(
            subject_id,
            fact,
            fact_category,
            FactSource::DmNarration,
            &mentioned_ids,
            importance,
        );
    }
}
