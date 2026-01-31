//! AI Dungeon Master agent.
//!
//! The DungeonMaster struct provides the main interface for AI-powered
//! D&D gameplay. It uses the Claude API to generate narrative responses
//! and tool calls that are resolved by the RulesEngine.

use super::memory::{DmMemory, FactCategory};
use super::relevance::{RelevanceChecker, RelevanceResult};
use super::story_memory::{
    ConsequenceSeverity, EntityType, FactCategory as StoryFactCategory, FactSource, StoryMemory,
};
use super::tools::{execute_info_tool, parse_tool_call, DmTools};
use crate::rules::{apply_effects, Effect, Intent, Resolution, RulesEngine};
use crate::world::{GameMode, GameWorld, NarrativeType};
use claude::{Claude, ContentBlock, Message, Request, StopReason, StreamEvent, ToolResult};
use futures::StreamExt;
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

        // Check for relevant consequences using fast model (Haiku)
        let relevance_result = self.check_relevance(player_input, world).await?;

        // Mark triggered consequences
        self.apply_relevance_results(&relevance_result);

        // Build system prompt with story context for this input
        let mut system_prompt = self.build_system_prompt(world, player_input);

        // Add triggered consequences to context
        let triggered_context = self.build_triggered_consequences_context(&relevance_result);
        if !triggered_context.is_empty() {
            system_prompt.push_str(&triggered_context);
        }

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
                // First check if it's an informational tool
                let result = if let Some(info_result) = execute_info_tool(&name, &input, world) {
                    // Info tools just return data without changing state
                    ToolResult::success(&info_result)
                } else if let Some(intent) = parse_tool_call(&name, &input, world) {
                    // Resolve the intent
                    let resolution = self.rules.resolve(world, intent.clone());

                    // Apply effects to world
                    apply_effects(world, &resolution.effects);

                    // Handle FactRemembered and ConsequenceRegistered effects specially - store in story memory
                    for effect in &resolution.effects {
                        match effect {
                            Effect::FactRemembered {
                                subject_name,
                                subject_type,
                                fact,
                                category,
                                related_entities,
                                importance,
                            } => {
                                self.store_fact(
                                    subject_name,
                                    subject_type,
                                    fact,
                                    category,
                                    related_entities,
                                    *importance,
                                );
                            }
                            Effect::ConsequenceRegistered {
                                trigger_description,
                                consequence_description,
                                severity,
                                ..
                            } => {
                                self.store_consequence(
                                    trigger_description,
                                    consequence_description,
                                    severity,
                                );
                            }
                            _ => {}
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

    /// Process player input with streaming text callbacks.
    ///
    /// The callback is called with each text delta as it arrives.
    /// Tool execution still happens synchronously between text chunks.
    pub async fn process_input_streaming<F>(
        &mut self,
        player_input: &str,
        world: &mut GameWorld,
        mut on_text: F,
    ) -> Result<DmResponse, DmError>
    where
        F: FnMut(&str) + Send,
    {
        // Advance story turn
        self.story_memory.advance_turn();

        // Add player input to memory
        self.memory.add_player_message(player_input);

        // Add to game world narrative
        world.add_narrative(player_input.to_string(), NarrativeType::PlayerAction);

        // Check for relevant consequences using fast model (Haiku)
        let relevance_result = self.check_relevance(player_input, world).await?;

        // Mark triggered consequences
        self.apply_relevance_results(&relevance_result);

        // Build system prompt with story context for this input
        let mut system_prompt = self.build_system_prompt(world, player_input);

        // Add triggered consequences to context
        let triggered_context = self.build_triggered_consequences_context(&relevance_result);
        if !triggered_context.is_empty() {
            system_prompt.push_str(&triggered_context);
        }

        // Track intents, effects, and resolutions
        let mut all_intents = Vec::new();
        let mut all_effects = Vec::new();
        let mut all_resolutions = Vec::new();
        let mut narrative = String::new();

        // Build initial messages
        let mut messages = self.memory.get_messages();

        // Tool use loop
        let mut iteration = 0;
        loop {
            // Add paragraph break between narrative from different API calls
            // (e.g., when continuing after tool results)
            if iteration > 0 && !narrative.is_empty() && !narrative.ends_with('\n') {
                narrative.push_str("\n\n");
                on_text("\n\n");
            }
            iteration += 1;

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

            // Use streaming API
            let mut stream = self.client.stream(request).await?;

            // Track tool uses being accumulated
            let mut tool_uses: Vec<PartialToolUse> = Vec::new();
            let mut current_tool_index: Option<usize> = None;
            let mut stop_reason = StopReason::EndTurn;

            while let Some(event_result) = stream.next().await {
                let event = event_result?;
                match event {
                    StreamEvent::TextDelta { text, .. } => {
                        // Send text to callback immediately
                        on_text(&text);
                        narrative.push_str(&text);
                    }
                    StreamEvent::ContentBlockStart {
                        index,
                        content_type,
                        tool_use_id,
                        tool_name,
                    } => {
                        if content_type == "tool_use" {
                            // Start accumulating a new tool use
                            current_tool_index = Some(index);
                            tool_uses.push(PartialToolUse {
                                id: tool_use_id.unwrap_or_default(),
                                name: tool_name.unwrap_or_default(),
                                json_buffer: String::new(),
                            });
                        }
                    }
                    StreamEvent::InputJsonDelta {
                        index,
                        partial_json,
                    } => {
                        // Accumulate JSON for the current tool use
                        if let Some(current_idx) = current_tool_index {
                            if index == current_idx {
                                if let Some(tool) = tool_uses.last_mut() {
                                    tool.json_buffer.push_str(&partial_json);
                                }
                            }
                        }
                    }
                    StreamEvent::ContentBlockStop { index } => {
                        // Reset current tool index if this was a tool block
                        if Some(index) == current_tool_index {
                            current_tool_index = None;
                        }
                    }
                    StreamEvent::MessageDelta {
                        stop_reason: Some(sr),
                    } => {
                        stop_reason = sr;
                    }
                    StreamEvent::Error { message } => {
                        return Err(DmError::ToolError(format!("Stream error: {message}")));
                    }
                    _ => {
                        // Ignore other events (MessageStart, MessageStop, Ping, etc.)
                    }
                }
            }

            // If no tool calls or stop reason isn't ToolUse, we're done
            if stop_reason != StopReason::ToolUse || tool_uses.is_empty() {
                break;
            }

            // Build assistant message content from what we received
            let mut assistant_content: Vec<ContentBlock> = Vec::new();

            // Add narrative text if any
            if !narrative.is_empty() {
                // Only add the new narrative since last iteration
                assistant_content.push(ContentBlock::Text {
                    text: narrative.clone(),
                });
            }

            // Add tool uses
            for tool in &tool_uses {
                // Parse JSON input, defaulting to empty object if parsing fails
                // (Claude API requires tool_use.input to be a valid dictionary)
                let input: serde_json::Value = serde_json::from_str(&tool.json_buffer)
                    .unwrap_or_else(|_| serde_json::json!({}));
                assistant_content.push(ContentBlock::ToolUse {
                    id: tool.id.clone(),
                    name: tool.name.clone(),
                    input,
                });
            }

            // Add assistant response to messages
            messages.push(Message {
                role: claude::Role::Assistant,
                content: assistant_content,
            });

            // Execute tools and collect results
            let mut tool_results = Vec::new();
            for tool in tool_uses {
                // Parse JSON input, defaulting to empty object if parsing fails
                let input: serde_json::Value = serde_json::from_str(&tool.json_buffer)
                    .unwrap_or_else(|_| serde_json::json!({}));

                // First check if it's an informational tool
                let result = if let Some(info_result) = execute_info_tool(&tool.name, &input, world)
                {
                    // Info tools just return data without changing state
                    ToolResult::success(&info_result)
                } else if let Some(intent) = parse_tool_call(&tool.name, &input, world) {
                    // Resolve the intent
                    let resolution = self.rules.resolve(world, intent.clone());

                    // Apply effects to world
                    apply_effects(world, &resolution.effects);

                    // Handle FactRemembered and ConsequenceRegistered effects specially - store in story memory
                    for effect in &resolution.effects {
                        match effect {
                            Effect::FactRemembered {
                                subject_name,
                                subject_type,
                                fact,
                                category,
                                related_entities,
                                importance,
                            } => {
                                self.store_fact(
                                    subject_name,
                                    subject_type,
                                    fact,
                                    category,
                                    related_entities,
                                    *importance,
                                );
                            }
                            Effect::ConsequenceRegistered {
                                trigger_description,
                                consequence_description,
                                severity,
                                ..
                            } => {
                                self.store_consequence(
                                    trigger_description,
                                    consequence_description,
                                    severity,
                                );
                            }
                            _ => {}
                        }
                    }

                    // Store for response
                    all_intents.push(intent);
                    all_effects.extend(resolution.effects.clone());
                    all_resolutions.push(resolution.clone());

                    // Return narrative as tool result
                    ToolResult::success(&resolution.narrative)
                } else {
                    ToolResult::error(format!("Unknown tool: {}", tool.name))
                };

                tool_results.push(ContentBlock::ToolResult {
                    tool_use_id: tool.id,
                    content: result.content,
                    is_error: result.is_error,
                });
            }

            // Add tool results as user message
            messages.push(Message {
                role: claude::Role::User,
                content: tool_results,
            });

            // Clear tool_uses for next iteration
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

        // Add class mechanical reference - detailed rules for class features
        prompt.push_str("\n\n");
        prompt.push_str(include_str!("prompts/class_reference.txt"));

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
            let class_info: Vec<_> = pc
                .classes
                .iter()
                .map(|c| format!("{} {}", c.class.name(), c.level))
                .collect();
            prompt.push_str(&format!(" ({})", class_info.join("/")));
        }
        prompt.push('\n');
        prompt.push_str(&format!("**Race:** {}\n", pc.race.name));
        prompt.push_str(&format!(
            "**Background:** {} - {}\n",
            pc.background.name(),
            pc.background.description()
        ));
        prompt.push_str(&format!(
            "**HP:** {}/{}\n",
            pc.hit_points.current, pc.hit_points.maximum
        ));
        prompt.push_str(&format!("**AC:** {}\n", pc.current_ac()));

        // Add backstory if present
        if let Some(ref backstory) = pc.backstory {
            prompt.push_str(&format!("\n**Backstory:**\n{}\n", backstory));
        }

        // Add spellcasting info if present
        if let Some(ref spellcasting) = pc.spellcasting {
            prompt.push_str("\n**Spellcasting:**\n");
            prompt.push_str(&format!(
                "- Ability: {} (DC {}, +{} to hit)\n",
                spellcasting.ability.name(),
                spellcasting.spell_save_dc(&pc.ability_scores, pc.proficiency_bonus()),
                spellcasting.spell_attack_bonus(&pc.ability_scores, pc.proficiency_bonus())
            ));
            if !spellcasting.cantrips_known.is_empty() {
                prompt.push_str(&format!(
                    "- Cantrips: {}\n",
                    spellcasting.cantrips_known.join(", ")
                ));
            }
            if !spellcasting.spells_prepared.is_empty() {
                prompt.push_str(&format!(
                    "- Prepared Spells: {}\n",
                    spellcasting.spells_prepared.join(", ")
                ));
            } else if !spellcasting.spells_known.is_empty() {
                prompt.push_str(&format!(
                    "- Known Spells: {}\n",
                    spellcasting.spells_known.join(", ")
                ));
            }
            // Show available spell slots
            let slots: Vec<String> = spellcasting
                .spell_slots
                .slots
                .iter()
                .enumerate()
                .filter(|(_, s)| s.total > 0)
                .map(|(i, s)| format!("L{}: {}/{}", i + 1, s.available(), s.total))
                .collect();
            if !slots.is_empty() {
                prompt.push_str(&format!("- Spell Slots: {}\n", slots.join(", ")));
            }
        }

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
        prompt.push_str(&format!(
            "Time: {} ({})\n",
            world.game_time.time_of_day(),
            if world.game_time.is_daytime() {
                "day"
            } else {
                "night"
            }
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

    /// Store a consequence in story memory.
    fn store_consequence(
        &mut self,
        trigger_description: &str,
        consequence_description: &str,
        severity: &str,
    ) {
        let severity_enum = match severity.to_lowercase().as_str() {
            "minor" => ConsequenceSeverity::Minor,
            "moderate" => ConsequenceSeverity::Moderate,
            "major" => ConsequenceSeverity::Major,
            "critical" => ConsequenceSeverity::Critical,
            _ => ConsequenceSeverity::Moderate,
        };

        self.story_memory.create_consequence(
            trigger_description,
            consequence_description,
            severity_enum,
        );
    }

    /// Check relevance of stored context against player input using a fast model.
    ///
    /// Returns triggered consequences and relevant entities that should be
    /// included in the DM's context.
    pub async fn check_relevance(
        &self,
        player_input: &str,
        world: &GameWorld,
    ) -> Result<RelevanceResult, DmError> {
        // Only check if we have pending consequences
        if self.story_memory.pending_consequence_count() == 0 {
            return Ok(RelevanceResult::default());
        }

        let checker = RelevanceChecker::new(self.client.clone());
        let result = checker
            .check_relevance(
                player_input,
                &world.current_location.name,
                &self.story_memory,
            )
            .await
            .map_err(|e| DmError::ToolError(format!("Relevance check failed: {e}")))?;

        Ok(result)
    }

    /// Mark consequences as triggered based on relevance check results.
    pub fn apply_relevance_results(&mut self, results: &RelevanceResult) {
        for consequence_id in &results.triggered_consequences {
            self.story_memory.trigger_consequence(*consequence_id);
        }
    }

    /// Build additional context for triggered consequences.
    fn build_triggered_consequences_context(&self, results: &RelevanceResult) -> String {
        if results.triggered_consequences.is_empty() {
            return String::new();
        }

        let mut context = String::new();
        context.push_str("\n## TRIGGERED CONSEQUENCES - ACT ON THESE!\n");
        context
            .push_str("The following consequences have been triggered by the player's action:\n\n");

        for id in &results.triggered_consequences {
            if let Some(consequence) = self.story_memory.get_consequence(*id) {
                context.push_str(&format!(
                    "- **{}** ({}): {}\n",
                    consequence.severity.name(),
                    consequence.trigger_description,
                    consequence.consequence_description
                ));
            }
        }

        context.push_str("\nYou MUST incorporate these consequences into your response!\n");
        context
    }
}

/// Helper for accumulating tool use data during streaming.
struct PartialToolUse {
    /// Tool use ID from the API.
    id: String,
    /// Tool name.
    name: String,
    /// Accumulated JSON input buffer.
    json_buffer: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dm::story_memory::ConsequenceId;
    use crate::world::{Character, CharacterClass, Location, LocationType};

    fn create_test_world() -> GameWorld {
        let mut character = Character::new("Test Hero");
        character.classes.push(crate::world::ClassLevel {
            class: CharacterClass::Fighter,
            level: 1,
            subclass: None,
        });
        let mut world = GameWorld::new("Test Campaign", character);
        world.current_location =
            Location::new("Test Location", LocationType::Town).with_description("A test location");
        world
    }

    #[test]
    fn test_dm_config_default() {
        let config = DmConfig::default();
        assert!(config.model.is_none());
        assert_eq!(config.max_tokens, 4096);
        assert_eq!(config.temperature, Some(0.8));
        assert!(config.custom_system_prompt.is_none());
    }

    #[test]
    fn test_dm_response_creation() {
        let response = DmResponse {
            narrative: "You enter the dark cave.".to_string(),
            intents: vec![],
            effects: vec![],
            resolutions: vec![],
        };
        assert_eq!(response.narrative, "You enter the dark cave.");
        assert!(response.intents.is_empty());
    }

    #[test]
    fn test_story_memory_access() {
        let dm = DungeonMaster::new("test-key");
        let memory = dm.story_memory();
        assert_eq!(memory.current_turn(), 0);
    }

    #[test]
    fn test_dm_memory_access() {
        let dm = DungeonMaster::new("test-key");
        let memory = dm.memory();
        assert!(memory.get_messages().is_empty());
    }

    #[test]
    fn test_with_config() {
        let config = DmConfig {
            model: Some("claude-sonnet-4-20250514".to_string()),
            max_tokens: 2048,
            temperature: Some(0.5),
            custom_system_prompt: Some("Custom prompt".to_string()),
        };

        let _dm = DungeonMaster::new("test-key").with_config(config);
    }

    #[test]
    fn test_build_system_prompt_contains_character_info() {
        let dm = DungeonMaster::new("test-key");
        let world = create_test_world();
        let prompt = dm.build_system_prompt(&world, "I look around");

        // The system prompt should contain character name
        assert!(prompt.contains("Test Hero"));
        // Should contain location
        assert!(prompt.contains("Test Location"));
    }

    #[test]
    fn test_relevance_result_triggers() {
        let result = RelevanceResult {
            triggered_consequences: vec![
                ConsequenceId::new(),
                ConsequenceId::new(),
                ConsequenceId::new(),
            ],
            relevant_facts: vec![],
            relevant_entities: vec![],
            explanation: None,
        };

        let dm = DungeonMaster::new("test-key");
        let context = dm.build_triggered_consequences_context(&result);

        // Should include header when there are triggered consequences
        assert!(context.contains("TRIGGERED CONSEQUENCES"));
    }

    #[test]
    fn test_empty_relevance_result() {
        let result = RelevanceResult {
            triggered_consequences: vec![],
            relevant_facts: vec![],
            relevant_entities: vec![],
            explanation: None,
        };

        let dm = DungeonMaster::new("test-key");
        let context = dm.build_triggered_consequences_context(&result);

        // Should be empty when no triggered consequences
        assert!(context.is_empty());
    }

    #[test]
    fn test_partial_tool_use_struct() {
        let partial = PartialToolUse {
            id: "tool_123".to_string(),
            name: "roll_dice".to_string(),
            json_buffer: r#"{"notation": "1d20"}"#.to_string(),
        };

        assert_eq!(partial.id, "tool_123");
        assert_eq!(partial.name, "roll_dice");
        assert!(partial.json_buffer.contains("1d20"));
    }
}
