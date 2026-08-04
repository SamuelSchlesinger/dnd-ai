# Changelog

All notable changes to chronicler are documented in this file.

## [Unreleased]

### Core Features
- **AI Dungeon Master** - Configurable LLM-powered DM that narrates adventures, runs NPCs, and adjudicates actions
- **D&D 5e Rules Engine** - Full implementation of ability checks, saving throws, combat, conditions, and death saves
- **Story Memory System** - AI remembers characters, locations, events, and relationships across sessions
- **Consequence System** - Actions have lasting effects that resurface naturally (using the provider's fast model for semantic relevance)
- **Bevy GUI** - Modern desktop application with medieval-themed dark UI
- **Character Creation Wizard** - Full SRD races, classes, backgrounds, and point-buy ability scores
- **Save/Load System** - Persist and continue campaigns across sessions
- **Complete Spellcasting** - Spell slots, spell lists, casting, and ritual casting for all SRD caster classes
- **Equipment & Inventory** - SRD weapons, armor, and adventuring gear with encumbrance tracking
- **Combat System** - Initiative, attack rolls, damage, conditions, and death saving throws

### Recent Features
- Add Anthropic, OpenAI, OpenRouter/Kimi K3, and Ollama provider support
- Add `--local [MODEL]`, defaulting to `qwen3.6:35b-a3b`
- Add deferred_effects config option to DM for controlling effect timing
- Persist audio settings to disk
- Add sound system with synthesized effects and real-time streaming
- Implement level-up mechanics and award_experience tool
- Add onboarding modal and spell tooltips for new players
- Add silver currency alongside gold
- Add Bardic Inspiration use tracking for Bard class
- Add clickable spell details in character sheet

### Improvements
- Replace Unicode characters with ASCII in UI for better compatibility
- Deduplicate point_buy_cost function
- Modularize codebase: split large files into submodules (ui/overlays, state, sound, effects, dm/tools, spells, world, LLM client)
- Remove unused animation code and play_time_minutes field
- Improve start_combat tool schema with enemy stat parameters
- Fix hardcoded combat stats for proper AC and initiative

### Bug Fixes
- Remove unicode emojis from onboarding modal
- Resolve all clippy warnings across workspace
- Fix three correctness bugs in D&D rules engine
- Fix condition duration tracking
- Fix rage mechanics, conditions, and spells issues from beta testing

### Tests
- Add unit tests for to_snake_case in chronicler-macros
- Add additional tests for DM memory and relevance
- Add QA test suite for code quality

### Documentation
- Add HOW_IT_WORKS.md explaining AI DM system
- Add example transcripts showcasing AI Dungeon Master
- Add SRD 5.2 attribution and licensing compliance guidelines
- Add contributor guide

### Infrastructure
- Add GitHub Actions CI workflow
- Add CC BY-NC 4.0 license for non-commercial use
