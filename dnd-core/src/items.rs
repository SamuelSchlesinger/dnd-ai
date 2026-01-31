//! Standard D&D 5e item database.
//!
//! Contains predefined items including weapons, armor, potions, and adventuring gear
//! that the DM can reference by name.

use crate::world::{
    ArmorItem, ArmorType, ConsumableEffect, ConsumableItem, Item, ItemType, WeaponDamageType,
    WeaponItem, WeaponProperty,
};

/// Get a standard weapon by name.
pub fn get_weapon(name: &str) -> Option<WeaponItem> {
    let name_lower = name.to_lowercase();
    WEAPONS
        .iter()
        .find(|w| w.base.name.to_lowercase() == name_lower)
        .cloned()
}

/// Get a standard armor piece by name.
pub fn get_armor(name: &str) -> Option<ArmorItem> {
    let name_lower = name.to_lowercase();
    ARMORS
        .iter()
        .find(|a| a.base.name.to_lowercase() == name_lower)
        .cloned()
}

/// Get a standard potion by name.
pub fn get_potion(name: &str) -> Option<ConsumableItem> {
    let name_lower = name.to_lowercase();
    POTIONS
        .iter()
        .find(|p| p.base.name.to_lowercase() == name_lower)
        .cloned()
}

/// Get a standard adventuring item by name.
pub fn get_adventuring_gear(name: &str) -> Option<Item> {
    let name_lower = name.to_lowercase();
    ADVENTURING_GEAR
        .iter()
        .find(|i| i.name.to_lowercase() == name_lower)
        .cloned()
}

/// Try to find any standard item by name.
pub fn find_item(name: &str) -> Option<StandardItem> {
    if let Some(weapon) = get_weapon(name) {
        return Some(StandardItem::Weapon(weapon));
    }
    if let Some(armor) = get_armor(name) {
        return Some(StandardItem::Armor(armor));
    }
    if let Some(potion) = get_potion(name) {
        return Some(StandardItem::Consumable(potion));
    }
    if let Some(item) = get_adventuring_gear(name) {
        return Some(StandardItem::Item(item));
    }
    None
}

/// A standard item from the database.
#[derive(Debug, Clone)]
pub enum StandardItem {
    Weapon(WeaponItem),
    Armor(ArmorItem),
    Consumable(ConsumableItem),
    Item(Item),
}

impl StandardItem {
    /// Get the base item for any standard item.
    pub fn as_item(&self) -> Item {
        match self {
            StandardItem::Weapon(w) => w.base.clone(),
            StandardItem::Armor(a) => a.base.clone(),
            StandardItem::Consumable(c) => c.base.clone(),
            StandardItem::Item(i) => i.clone(),
        }
    }
}

// ============================================================================
// Weapons
// ============================================================================

lazy_static::lazy_static! {
    /// Standard D&D 5e weapons.
    pub static ref WEAPONS: Vec<WeaponItem> = vec![
        // Simple Melee Weapons
        WeaponItem::new("Club", "1d4", WeaponDamageType::Bludgeoning)
            .with_weight(2.0)
            .with_value(0.1)
            .with_properties(vec![WeaponProperty::Light]),
        WeaponItem::new("Dagger", "1d4", WeaponDamageType::Piercing)
            .with_weight(1.0)
            .with_value(2.0)
            .with_properties(vec![WeaponProperty::Finesse, WeaponProperty::Light, WeaponProperty::Thrown])
            .with_range(20, 60),
        WeaponItem::new("Greatclub", "1d8", WeaponDamageType::Bludgeoning)
            .with_weight(10.0)
            .with_value(0.2)
            .with_properties(vec![WeaponProperty::TwoHanded]),
        WeaponItem::new("Handaxe", "1d6", WeaponDamageType::Slashing)
            .with_weight(2.0)
            .with_value(5.0)
            .with_properties(vec![WeaponProperty::Light, WeaponProperty::Thrown])
            .with_range(20, 60),
        WeaponItem::new("Javelin", "1d6", WeaponDamageType::Piercing)
            .with_weight(2.0)
            .with_value(0.5)
            .with_properties(vec![WeaponProperty::Thrown])
            .with_range(30, 120),
        WeaponItem::new("Light Hammer", "1d4", WeaponDamageType::Bludgeoning)
            .with_weight(2.0)
            .with_value(2.0)
            .with_properties(vec![WeaponProperty::Light, WeaponProperty::Thrown])
            .with_range(20, 60),
        WeaponItem::new("Mace", "1d6", WeaponDamageType::Bludgeoning)
            .with_weight(4.0)
            .with_value(5.0),
        WeaponItem::new("Quarterstaff", "1d6", WeaponDamageType::Bludgeoning)
            .with_weight(4.0)
            .with_value(0.2)
            .with_properties(vec![WeaponProperty::Versatile("1d8".to_string())]),
        WeaponItem::new("Sickle", "1d4", WeaponDamageType::Slashing)
            .with_weight(2.0)
            .with_value(1.0)
            .with_properties(vec![WeaponProperty::Light]),
        WeaponItem::new("Spear", "1d6", WeaponDamageType::Piercing)
            .with_weight(3.0)
            .with_value(1.0)
            .with_properties(vec![WeaponProperty::Thrown, WeaponProperty::Versatile("1d8".to_string())])
            .with_range(20, 60),

        // Martial Melee Weapons
        WeaponItem::new("Battleaxe", "1d8", WeaponDamageType::Slashing)
            .with_weight(4.0)
            .with_value(10.0)
            .with_properties(vec![WeaponProperty::Versatile("1d10".to_string())]),
        WeaponItem::new("Flail", "1d8", WeaponDamageType::Bludgeoning)
            .with_weight(2.0)
            .with_value(10.0),
        WeaponItem::new("Glaive", "1d10", WeaponDamageType::Slashing)
            .with_weight(6.0)
            .with_value(20.0)
            .with_properties(vec![WeaponProperty::Heavy, WeaponProperty::Reach, WeaponProperty::TwoHanded]),
        WeaponItem::new("Greataxe", "1d12", WeaponDamageType::Slashing)
            .with_weight(7.0)
            .with_value(30.0)
            .with_properties(vec![WeaponProperty::Heavy, WeaponProperty::TwoHanded]),
        WeaponItem::new("Greatsword", "2d6", WeaponDamageType::Slashing)
            .with_weight(6.0)
            .with_value(50.0)
            .with_properties(vec![WeaponProperty::Heavy, WeaponProperty::TwoHanded]),
        WeaponItem::new("Halberd", "1d10", WeaponDamageType::Slashing)
            .with_weight(6.0)
            .with_value(20.0)
            .with_properties(vec![WeaponProperty::Heavy, WeaponProperty::Reach, WeaponProperty::TwoHanded]),
        WeaponItem::new("Lance", "1d12", WeaponDamageType::Piercing)
            .with_weight(6.0)
            .with_value(10.0)
            .with_properties(vec![WeaponProperty::Reach]),
        WeaponItem::new("Longsword", "1d8", WeaponDamageType::Slashing)
            .with_weight(3.0)
            .with_value(15.0)
            .with_properties(vec![WeaponProperty::Versatile("1d10".to_string())]),
        WeaponItem::new("Maul", "2d6", WeaponDamageType::Bludgeoning)
            .with_weight(10.0)
            .with_value(10.0)
            .with_properties(vec![WeaponProperty::Heavy, WeaponProperty::TwoHanded]),
        WeaponItem::new("Morningstar", "1d8", WeaponDamageType::Piercing)
            .with_weight(4.0)
            .with_value(15.0),
        WeaponItem::new("Pike", "1d10", WeaponDamageType::Piercing)
            .with_weight(18.0)
            .with_value(5.0)
            .with_properties(vec![WeaponProperty::Heavy, WeaponProperty::Reach, WeaponProperty::TwoHanded]),
        WeaponItem::new("Rapier", "1d8", WeaponDamageType::Piercing)
            .with_weight(2.0)
            .with_value(25.0)
            .with_properties(vec![WeaponProperty::Finesse]),
        WeaponItem::new("Scimitar", "1d6", WeaponDamageType::Slashing)
            .with_weight(3.0)
            .with_value(25.0)
            .with_properties(vec![WeaponProperty::Finesse, WeaponProperty::Light]),
        WeaponItem::new("Shortsword", "1d6", WeaponDamageType::Piercing)
            .with_weight(2.0)
            .with_value(10.0)
            .with_properties(vec![WeaponProperty::Finesse, WeaponProperty::Light]),
        WeaponItem::new("Trident", "1d6", WeaponDamageType::Piercing)
            .with_weight(4.0)
            .with_value(5.0)
            .with_properties(vec![WeaponProperty::Thrown, WeaponProperty::Versatile("1d8".to_string())])
            .with_range(20, 60),
        WeaponItem::new("War Pick", "1d8", WeaponDamageType::Piercing)
            .with_weight(2.0)
            .with_value(5.0),
        WeaponItem::new("Warhammer", "1d8", WeaponDamageType::Bludgeoning)
            .with_weight(2.0)
            .with_value(15.0)
            .with_properties(vec![WeaponProperty::Versatile("1d10".to_string())]),
        WeaponItem::new("Whip", "1d4", WeaponDamageType::Slashing)
            .with_weight(3.0)
            .with_value(2.0)
            .with_properties(vec![WeaponProperty::Finesse, WeaponProperty::Reach]),

        // Simple Ranged Weapons
        WeaponItem::new("Light Crossbow", "1d8", WeaponDamageType::Piercing)
            .with_weight(5.0)
            .with_value(25.0)
            .with_properties(vec![WeaponProperty::Ammunition, WeaponProperty::Loading, WeaponProperty::TwoHanded])
            .with_range(80, 320),
        WeaponItem::new("Shortbow", "1d6", WeaponDamageType::Piercing)
            .with_weight(2.0)
            .with_value(25.0)
            .with_properties(vec![WeaponProperty::Ammunition, WeaponProperty::TwoHanded])
            .with_range(80, 320),

        // Martial Ranged Weapons
        WeaponItem::new("Hand Crossbow", "1d6", WeaponDamageType::Piercing)
            .with_weight(3.0)
            .with_value(75.0)
            .with_properties(vec![WeaponProperty::Ammunition, WeaponProperty::Light, WeaponProperty::Loading])
            .with_range(30, 120),
        WeaponItem::new("Heavy Crossbow", "1d10", WeaponDamageType::Piercing)
            .with_weight(18.0)
            .with_value(50.0)
            .with_properties(vec![WeaponProperty::Ammunition, WeaponProperty::Heavy, WeaponProperty::Loading, WeaponProperty::TwoHanded])
            .with_range(100, 400),
        WeaponItem::new("Longbow", "1d8", WeaponDamageType::Piercing)
            .with_weight(2.0)
            .with_value(50.0)
            .with_properties(vec![WeaponProperty::Ammunition, WeaponProperty::Heavy, WeaponProperty::TwoHanded])
            .with_range(150, 600),
    ];

    /// Standard D&D 5e armor.
    pub static ref ARMORS: Vec<ArmorItem> = vec![
        // Light Armor
        ArmorItem::new("Padded Armor", ArmorType::Light, 11)
            .with_weight(8.0)
            .with_value(5.0)
            .with_stealth_disadvantage(),
        ArmorItem::new("Leather Armor", ArmorType::Light, 11)
            .with_weight(10.0)
            .with_value(10.0),
        ArmorItem::new("Studded Leather", ArmorType::Light, 12)
            .with_weight(13.0)
            .with_value(45.0),

        // Medium Armor
        ArmorItem::new("Hide Armor", ArmorType::Medium, 12)
            .with_weight(12.0)
            .with_value(10.0),
        ArmorItem::new("Chain Shirt", ArmorType::Medium, 13)
            .with_weight(20.0)
            .with_value(50.0),
        ArmorItem::new("Scale Mail", ArmorType::Medium, 14)
            .with_weight(45.0)
            .with_value(50.0)
            .with_stealth_disadvantage(),
        ArmorItem::new("Breastplate", ArmorType::Medium, 14)
            .with_weight(20.0)
            .with_value(400.0),
        ArmorItem::new("Half Plate", ArmorType::Medium, 15)
            .with_weight(40.0)
            .with_value(750.0)
            .with_stealth_disadvantage(),

        // Heavy Armor
        ArmorItem::new("Ring Mail", ArmorType::Heavy, 14)
            .with_weight(40.0)
            .with_value(30.0)
            .with_stealth_disadvantage(),
        ArmorItem::new("Chain Mail", ArmorType::Heavy, 16)
            .with_weight(55.0)
            .with_value(75.0)
            .with_strength_requirement(13)
            .with_stealth_disadvantage(),
        ArmorItem::new("Splint Armor", ArmorType::Heavy, 17)
            .with_weight(60.0)
            .with_value(200.0)
            .with_strength_requirement(15)
            .with_stealth_disadvantage(),
        ArmorItem::new("Plate Armor", ArmorType::Heavy, 18)
            .with_weight(65.0)
            .with_value(1500.0)
            .with_strength_requirement(15)
            .with_stealth_disadvantage(),
    ];

    /// Standard D&D 5e potions.
    pub static ref POTIONS: Vec<ConsumableItem> = vec![
        ConsumableItem::healing_potion("Potion of Healing", "2d4", 2, 50.0),
        ConsumableItem::healing_potion("Potion of Greater Healing", "4d4", 4, 150.0),
        ConsumableItem::healing_potion("Potion of Superior Healing", "8d4", 8, 450.0),
        ConsumableItem::healing_potion("Potion of Supreme Healing", "10d4", 20, 1350.0),
        ConsumableItem {
            base: Item {
                name: "Antitoxin".to_string(),
                quantity: 1,
                weight: 0.0,
                value_gp: 50.0,
                description: Some("Grants advantage on saving throws against poison for 1 hour.".to_string()),
                item_type: ItemType::Potion,
                magical: false,
            },
            effect: ConsumableEffect::GrantAdvantage {
                roll_type: "poison saves".to_string(),
                duration_rounds: 600, // ~1 hour
            },
        },
    ];

    /// Standard adventuring gear.
    pub static ref ADVENTURING_GEAR: Vec<Item> = vec![
        Item {
            name: "Backpack".to_string(),
            quantity: 1,
            weight: 5.0,
            value_gp: 2.0,
            description: Some("A leather pack for carrying gear.".to_string()),
            item_type: ItemType::Adventuring,
            magical: false,
        },
        Item {
            name: "Bedroll".to_string(),
            quantity: 1,
            weight: 7.0,
            value_gp: 1.0,
            description: None,
            item_type: ItemType::Adventuring,
            magical: false,
        },
        Item {
            name: "Rope (50 feet)".to_string(),
            quantity: 1,
            weight: 10.0,
            value_gp: 1.0,
            description: Some("Hemp rope, 50 feet.".to_string()),
            item_type: ItemType::Adventuring,
            magical: false,
        },
        Item {
            name: "Torch".to_string(),
            quantity: 1,
            weight: 1.0,
            value_gp: 0.01,
            description: Some("Provides bright light in 20-foot radius, dim light for 20 feet beyond. Burns for 1 hour.".to_string()),
            item_type: ItemType::Adventuring,
            magical: false,
        },
        Item {
            name: "Rations (1 day)".to_string(),
            quantity: 1,
            weight: 2.0,
            value_gp: 0.5,
            description: Some("Trail rations for one day.".to_string()),
            item_type: ItemType::Adventuring,
            magical: false,
        },
        Item {
            name: "Waterskin".to_string(),
            quantity: 1,
            weight: 5.0,
            value_gp: 0.2,
            description: Some("Holds 4 pints of liquid.".to_string()),
            item_type: ItemType::Adventuring,
            magical: false,
        },
        Item {
            name: "Tinderbox".to_string(),
            quantity: 1,
            weight: 1.0,
            value_gp: 0.5,
            description: Some("Used to light fires.".to_string()),
            item_type: ItemType::Adventuring,
            magical: false,
        },
        Item {
            name: "Lantern".to_string(),
            quantity: 1,
            weight: 2.0,
            value_gp: 5.0,
            description: Some("A hooded lantern casts bright light in 30-foot radius.".to_string()),
            item_type: ItemType::Adventuring,
            magical: false,
        },
        Item {
            name: "Oil Flask".to_string(),
            quantity: 1,
            weight: 1.0,
            value_gp: 0.1,
            description: Some("Flask of oil for lanterns or as improvised weapon.".to_string()),
            item_type: ItemType::Adventuring,
            magical: false,
        },
        Item {
            name: "Grappling Hook".to_string(),
            quantity: 1,
            weight: 4.0,
            value_gp: 2.0,
            description: None,
            item_type: ItemType::Adventuring,
            magical: false,
        },
        Item {
            name: "Crowbar".to_string(),
            quantity: 1,
            weight: 5.0,
            value_gp: 2.0,
            description: Some("Grants advantage on Strength checks to pry things open.".to_string()),
            item_type: ItemType::Tool,
            magical: false,
        },
        Item {
            name: "Thieves' Tools".to_string(),
            quantity: 1,
            weight: 1.0,
            value_gp: 25.0,
            description: Some("Required for picking locks and disarming traps.".to_string()),
            item_type: ItemType::Tool,
            magical: false,
        },
        Item {
            name: "Holy Symbol".to_string(),
            quantity: 1,
            weight: 1.0,
            value_gp: 5.0,
            description: Some("A religious focus for spellcasting.".to_string()),
            item_type: ItemType::Adventuring,
            magical: false,
        },
        Item {
            name: "Arcane Focus".to_string(),
            quantity: 1,
            weight: 1.0,
            value_gp: 10.0,
            description: Some("A crystal, orb, or similar item used as a spellcasting focus.".to_string()),
            item_type: ItemType::Adventuring,
            magical: false,
        },
        Item {
            name: "Component Pouch".to_string(),
            quantity: 1,
            weight: 2.0,
            value_gp: 25.0,
            description: Some("A small pouch containing spell components.".to_string()),
            item_type: ItemType::Adventuring,
            magical: false,
        },
        Item {
            name: "Arrows (20)".to_string(),
            quantity: 1,
            weight: 1.0,
            value_gp: 1.0,
            description: Some("A quiver of 20 arrows.".to_string()),
            item_type: ItemType::Adventuring,
            magical: false,
        },
        Item {
            name: "Bolts (20)".to_string(),
            quantity: 1,
            weight: 1.5,
            value_gp: 1.0,
            description: Some("A case of 20 crossbow bolts.".to_string()),
            item_type: ItemType::Adventuring,
            magical: false,
        },
        Item {
            name: "Shield".to_string(),
            quantity: 1,
            weight: 6.0,
            value_gp: 10.0,
            description: Some("A wooden or metal shield. +2 AC when equipped.".to_string()),
            item_type: ItemType::Shield,
            magical: false,
        },
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_weapon() {
        let longsword = get_weapon("Longsword").unwrap();
        assert_eq!(longsword.damage_dice, "1d8");
        assert_eq!(longsword.damage_type, WeaponDamageType::Slashing);

        // Case insensitive
        let dagger = get_weapon("dagger").unwrap();
        assert_eq!(dagger.damage_dice, "1d4");
    }

    #[test]
    fn test_get_armor() {
        let plate = get_armor("Plate Armor").unwrap();
        assert_eq!(plate.base_ac, 18);
        assert!(matches!(plate.armor_type, ArmorType::Heavy));
    }

    #[test]
    fn test_get_potion() {
        let potion = get_potion("Potion of Healing").unwrap();
        assert!(matches!(potion.effect, ConsumableEffect::Healing { .. }));
    }

    #[test]
    fn test_find_item() {
        assert!(matches!(
            find_item("Longsword"),
            Some(StandardItem::Weapon(_))
        ));
        assert!(matches!(
            find_item("Chain Mail"),
            Some(StandardItem::Armor(_))
        ));
        assert!(matches!(
            find_item("Potion of Healing"),
            Some(StandardItem::Consumable(_))
        ));
        assert!(matches!(
            find_item("Rope (50 feet)"),
            Some(StandardItem::Item(_))
        ));
        assert!(find_item("Nonexistent Item").is_none());
    }
}
