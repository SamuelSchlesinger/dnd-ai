# How the AI Dungeon Master Works

*A peek behind the screen for the curious.*

---

The AI Dungeon Master isn't just a chatbot with a fantasy skin. It's a system designed to run actual D&D 5e — with proper rules, persistent memory, and narrative craft. Here's how.

## The Prompt Stack

Every time you take an action, the AI receives a carefully constructed context:

| Layer | Purpose |
|-------|---------|
| **Base DM Persona** | Narrative voice, pacing guidelines, NPC techniques |
| **Rules Reference** | D&D 5e mechanics — skill checks, combat, conditions, spells |
| **Story Memory** | Facts about NPCs, locations, and your past decisions |
| **Current State** | Your character sheet, active conditions, combat status |

The base prompt teaches the AI to be a *good* DM — not just one that knows the rules, but one that creates atmosphere, voices NPCs distinctively, and keeps the pace engaging.

## Tools, Not Just Words

The AI doesn't just describe what happens — it calls **tools** for mechanical resolution:

- **`skill_check`** / **`saving_throw`** — Proper D&D dice rolls with difficulty classes
- **`apply_damage`** / **`apply_healing`** — HP tracking that persists
- **`start_combat`** — Initiative rolls and turn order management
- **`roll_dice`** — Any dice expression: `2d6+3`, `4d6kh3`, `1d20 advantage`
- **`apply_condition`** — Poisoned, frightened, paralyzed, etc.
- **`remember_fact`** — Write to persistent story memory

This separation is important: the AI handles narrative creativity while a rules engine ensures mechanical consistency. The AI can't "forget" to apply your -2 penalty or accidentally give you more HP than you should have.

When you see output like:

> Rolling Stealth... 14 vs DC 12 — success!

That's a real roll. The tool returned 14, the DC was set by context, and the rules engine determined the outcome.

## Story Memory

Large language models have a problem: limited context windows. After roughly 20 exchanges, earlier details start falling out of working memory. In a long campaign, the AI would forget NPCs it introduced, decisions you made, and secrets it revealed.

The solution is **persistent story memory**. When the DM introduces something worth remembering, it writes a note:

```
remember_fact(
  subject_name: "Mira",
  subject_type: "npc",
  fact: "Nervous half-elf herbalist who stutters when lying.
        Sold the player healing herbs. Seemed scared when
        asked about the missing merchant.",
  importance: 0.7,
  related_entities: ["The Green Leaf Shop"]
)
```

These facts are stored in your save file. When you mention "Mira" or visit "The Green Leaf Shop" 50 turns later, the relevant memories are automatically injected into the AI's context.

The result: NPCs remember you. Consequences resurface. The world feels consistent even across multiple play sessions.

## Key Design Decisions

### Act Decisively

Bad AI DMs constantly ask clarifying questions:

> **Player:** "I attack whatever looks threatening."
> **Bad DM:** "Which enemy would you like to attack?"

This kills momentum. The dnd-ai DM is instructed to **pick the obvious choice and act**:

> **Player:** "I attack whatever looks threatening."
> **Good DM:** "You lunge at the Bandit Leader — clearly the biggest threat! Rolling attack... 18 vs AC 15 — hit! Rolling damage... 8 slashing damage! He staggers back, blood seeping through his leather armor."

Clarification is reserved for genuinely ambiguous situations with meaningful strategic differences.

### Intent/Effect Pattern

The AI generates **intents** ("attack the goblin with my sword"). The rules engine validates them and produces **effects** ("8 damage, goblin HP reduced to 3"). This means:

- The AI can't accidentally apply impossible damage
- All effects are logged and can be replayed
- Game state stays mechanically consistent even when the AI gets creative

### Gardener, Not Architect

The DM is prompted to be a "gardener" — planting story seeds and letting them grow based on your choices, rather than railroading you through a predetermined plot.

If you spare the bandit, that gets recorded. The bandit might return — as an ally, a recurring enemy, or a plot complication. The DM doesn't plan this in advance; it emerges from the memory system and the AI's narrative instincts.

## What This Enables

- **Multi-session campaigns** that remember everything
- **Mechanically correct** D&D 5e (not just vibes)
- **Emergent storytelling** from your choices
- **Distinctive NPCs** with consistent personalities
- **Proper pacing** — the AI knows when to push forward vs. linger

---

## Further Reading

- **[ARCHITECTURE.md](../ARCHITECTURE.md)** — Technical deep-dive into the codebase
- **[Example Transcripts](transcripts/)** — See the AI DM in action
- **[SRD 5.2](https://dnd.wizards.com/resources/systems-reference-document)** — The D&D rules we implement
