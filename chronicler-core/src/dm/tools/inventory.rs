//! Inventory and currency management tools.

use chronicler_llm::Tool;
use serde_json::json;

/// Give an item to the player.
pub fn give_item() -> Tool {
    Tool {
        name: "give_item".to_string(),
        description: "Give an item to the player. Use this when they find loot, receive rewards, or purchase items.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "item_name": {
                    "type": "string",
                    "description": "Name of the item (e.g., 'Longsword', 'Healing Potion', 'Rope')"
                },
                "quantity": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Number of items to give (default 1)"
                },
                "item_type": {
                    "type": "string",
                    "enum": ["weapon", "armor", "shield", "potion", "scroll", "wand", "ring", "wondrous", "adventuring", "tool", "other"],
                    "description": "Type of item"
                },
                "description": {
                    "type": "string",
                    "description": "Optional description of the item"
                },
                "magical": {
                    "type": "boolean",
                    "description": "Whether the item is magical (default false)"
                },
                "weight": {
                    "type": "number",
                    "description": "Weight in pounds (optional)"
                },
                "value_gp": {
                    "type": "number",
                    "description": "Value in gold pieces (optional)"
                }
            },
            "required": ["item_name"]
        }),
    }
}

/// Remove an item from the player's inventory.
pub fn remove_item() -> Tool {
    Tool {
        name: "remove_item".to_string(),
        description: "Remove an item from the player's inventory. Use when items are consumed, lost, sold, or given away.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "item_name": {
                    "type": "string",
                    "description": "Name of the item to remove"
                },
                "quantity": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Number of items to remove (default 1)"
                }
            },
            "required": ["item_name"]
        }),
    }
}

/// Use a consumable item from inventory.
pub fn use_item() -> Tool {
    Tool {
        name: "use_item".to_string(),
        description: "Use a consumable item from inventory. Handles potions (healing), scrolls (spells), and other consumables.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "item_name": {
                    "type": "string",
                    "description": "Name of the item to use"
                },
                "target": {
                    "type": "string",
                    "enum": ["self", "ally", "enemy"],
                    "description": "Target of the item effect (default 'self')"
                }
            },
            "required": ["item_name"]
        }),
    }
}

/// Equip a weapon, armor, or shield from inventory.
pub fn equip_item() -> Tool {
    Tool {
        name: "equip_item".to_string(),
        description:
            "Equip a weapon, armor, or shield from inventory. Affects AC and attack damage."
                .to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "item_name": {
                    "type": "string",
                    "description": "Name of the item to equip"
                }
            },
            "required": ["item_name"]
        }),
    }
}

/// Unequip an item from a slot.
pub fn unequip_item() -> Tool {
    Tool {
        name: "unequip_item".to_string(),
        description: "Unequip an item from a slot and return it to inventory.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "slot": {
                    "type": "string",
                    "enum": ["armor", "shield", "main_hand", "off_hand"],
                    "description": "Equipment slot to unequip from"
                }
            },
            "required": ["slot"]
        }),
    }
}

/// Add or remove gold pieces.
pub fn adjust_gold() -> Tool {
    Tool {
        name: "adjust_gold".to_string(),
        description: "Add or remove gold pieces (gp) from the player. Use this whenever the player receives or spends gold. 1 gp = 10 sp.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "amount": {
                    "type": "integer",
                    "description": "Amount of gold pieces to add (positive) or remove (negative)"
                },
                "reason": {
                    "type": "string",
                    "description": "Reason for the gold change (e.g., 'looting chest', 'buying supplies', 'quest reward')"
                }
            },
            "required": ["amount"]
        }),
    }
}

/// Add or remove silver pieces.
pub fn adjust_silver() -> Tool {
    Tool {
        name: "adjust_silver".to_string(),
        description: "Add or remove silver pieces (sp) from the player. Use this whenever the player receives or spends silver. 10 sp = 1 gp.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "amount": {
                    "type": "integer",
                    "description": "Amount of silver pieces to add (positive) or remove (negative)"
                },
                "reason": {
                    "type": "string",
                    "description": "Reason for the silver change (e.g., 'tip from traveler', 'buying ale', 'found in pocket')"
                }
            },
            "required": ["amount"]
        }),
    }
}

/// Display the player's current inventory.
pub fn show_inventory() -> Tool {
    Tool {
        name: "show_inventory".to_string(),
        description: "Display the player's current inventory, equipment, gold, and silver. Use this to check what items and currency they have.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
    }
}
