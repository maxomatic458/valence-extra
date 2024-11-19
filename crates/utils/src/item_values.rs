use valence::{prelude::Equipment, ItemKind};

pub trait EquipmentExt {
    /// The armor points of the equipment.
    fn armor_points(&self) -> f32;
    /// The armor toughness of the equipment.
    fn armor_toughness(&self) -> f32;
    /// The knockback resistance of the equipment.
    ///
    /// This is a value between 0.0 and 1.0.
    ///
    /// https://minecraft.wiki/w/Knockback_(mechanic)#Natural_knockback_resistance
    fn knockback_resistance(&self) -> f32;
}

impl EquipmentExt for Equipment {
    fn armor_points(&self) -> f32 {
        self.head().item.armor_points()
            + self.chest().item.armor_points()
            + self.legs().item.armor_points()
            + self.feet().item.armor_points()
    }

    fn armor_toughness(&self) -> f32 {
        self.head().item.armor_toughness()
            + self.chest().item.armor_toughness()
            + self.legs().item.armor_toughness()
            + self.feet().item.armor_toughness()
    }

    fn knockback_resistance(&self) -> f32 {
        self.head().item.knockback_resistance()
            + self.chest().item.knockback_resistance()
            + self.legs().item.knockback_resistance()
            + self.feet().item.knockback_resistance()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CombatSystem {
    Old,
    New,
}

pub trait ItemKindExt {
    /// The armor points of the item.
    fn armor_points(&self) -> f32;
    /// The armor toughness of the item.
    fn armor_toughness(&self) -> f32;
    /// The attack damage of the item.
    fn attack_damage(&self, combat_system: &CombatSystem) -> f32;
    /// The attack speed of the item stack, these values are only for [`CombatSystem::New`].
    fn attack_speed(&self) -> f32;
    /// The knockback resistance of the item.
    fn knockback_resistance(&self) -> f32;
}

impl ItemKindExt for ItemKind {
    fn armor_points(&self) -> f32 {
        match self {
            ItemKind::LeatherHelmet => 1.0,
            ItemKind::LeatherChestplate => 3.0,
            ItemKind::LeatherLeggings => 2.0,
            ItemKind::LeatherBoots => 1.0,

            ItemKind::ChainmailHelmet => 2.0,
            ItemKind::ChainmailChestplate => 5.0,
            ItemKind::ChainmailLeggings => 4.0,
            ItemKind::ChainmailBoots => 1.0,

            ItemKind::IronHelmet => 2.0,
            ItemKind::IronChestplate => 6.0,
            ItemKind::IronLeggings => 5.0,
            ItemKind::IronBoots => 2.0,

            ItemKind::GoldenHelmet => 2.0,
            ItemKind::GoldenChestplate => 5.0,
            ItemKind::GoldenLeggings => 3.0,
            ItemKind::GoldenBoots => 1.0,

            ItemKind::DiamondHelmet => 3.0,
            ItemKind::DiamondChestplate => 8.0,
            ItemKind::DiamondLeggings => 6.0,
            ItemKind::DiamondBoots => 3.0,

            ItemKind::NetheriteHelmet => 3.0,
            ItemKind::NetheriteChestplate => 8.0,
            ItemKind::NetheriteLeggings => 6.0,
            ItemKind::NetheriteBoots => 3.0,
            _ => 0.0,
        }
    }

    fn armor_toughness(&self) -> f32 {
        match self {
            ItemKind::DiamondHelmet => 2.0,
            ItemKind::DiamondChestplate => 2.0,
            ItemKind::DiamondLeggings => 2.0,
            ItemKind::DiamondBoots => 2.0,

            ItemKind::NetheriteHelmet => 3.0,
            ItemKind::NetheriteChestplate => 3.0,
            ItemKind::NetheriteLeggings => 3.0,
            ItemKind::NetheriteBoots => 3.0,
            _ => 0.0,
        }
    }

    fn attack_damage(&self, combat_system: &CombatSystem) -> f32 {
        match combat_system {
            CombatSystem::Old => match self {
                ItemKind::WoodenSword => 4.0,
                ItemKind::WoodenPickaxe => 2.0,
                ItemKind::WoodenHoe => 0.5,
                ItemKind::WoodenShovel => 1.0,
                ItemKind::WoodenAxe => 3.0,

                ItemKind::StoneSword => 5.0,
                ItemKind::StonePickaxe => 3.0,
                ItemKind::StoneHoe => 0.5,
                ItemKind::StoneShovel => 2.0,
                ItemKind::StoneAxe => 4.0,

                ItemKind::IronSword => 6.0,
                ItemKind::IronPickaxe => 4.0,
                ItemKind::IronHoe => 0.5,
                ItemKind::IronShovel => 3.0,
                ItemKind::IronAxe => 5.0,

                ItemKind::GoldenSword => 4.0,
                ItemKind::GoldenPickaxe => 2.0,
                ItemKind::GoldenHoe => 0.5,
                ItemKind::GoldenShovel => 1.0,
                ItemKind::GoldenAxe => 3.0,

                ItemKind::DiamondSword => 7.0,
                ItemKind::DiamondPickaxe => 5.0,
                ItemKind::DiamondHoe => 0.5,
                ItemKind::DiamondShovel => 4.0,
                ItemKind::DiamondAxe => 6.0,

                ItemKind::NetheriteSword => 8.0,
                ItemKind::NetheritePickaxe => 6.0,
                ItemKind::NetheriteHoe => 1.0,
                ItemKind::NetheriteShovel => 6.5,
                ItemKind::NetheriteAxe => 7.0,

                ItemKind::Trident => 9.0,

                _ => 1.0,
            },
            CombatSystem::New => match self {
                ItemKind::WoodenSword => 4.0,
                ItemKind::WoodenPickaxe => 2.0,
                ItemKind::WoodenHoe => 1.0,
                ItemKind::WoodenShovel => 2.5,
                ItemKind::WoodenAxe => 7.0,

                ItemKind::StoneSword => 5.0,
                ItemKind::StonePickaxe => 3.0,
                ItemKind::StoneHoe => 1.0,
                ItemKind::StoneShovel => 3.5,
                ItemKind::StoneAxe => 9.0,

                ItemKind::IronSword => 6.0,
                ItemKind::IronPickaxe => 4.0,
                ItemKind::IronHoe => 1.0,
                ItemKind::IronShovel => 4.5,
                ItemKind::IronAxe => 9.0,

                ItemKind::GoldenSword => 4.0,
                ItemKind::GoldenPickaxe => 2.0,
                ItemKind::GoldenHoe => 1.0,
                ItemKind::GoldenShovel => 2.5,
                ItemKind::GoldenAxe => 7.0,

                ItemKind::DiamondSword => 7.0,
                ItemKind::DiamondPickaxe => 5.0,
                ItemKind::DiamondHoe => 1.0,
                ItemKind::DiamondShovel => 5.5,
                ItemKind::DiamondAxe => 9.0,

                ItemKind::NetheriteSword => 8.0,
                ItemKind::NetheritePickaxe => 6.0,
                ItemKind::NetheriteHoe => 1.0,
                ItemKind::NetheriteShovel => 6.5,
                ItemKind::NetheriteAxe => 10.0,

                ItemKind::Trident => 9.0,
                _ => 1.0,
            },
        }
    }

    fn attack_speed(&self) -> f32 {
        match self {
            ItemKind::WoodenSword
            | ItemKind::StoneSword
            | ItemKind::IronSword
            | ItemKind::GoldenSword
            | ItemKind::DiamondSword
            | ItemKind::NetheriteSword => 1.6,

            ItemKind::WoodenShovel
            | ItemKind::StoneShovel
            | ItemKind::IronShovel
            | ItemKind::GoldenShovel
            | ItemKind::DiamondShovel
            | ItemKind::NetheriteShovel => 1.0,

            ItemKind::WoodenPickaxe
            | ItemKind::StonePickaxe
            | ItemKind::IronPickaxe
            | ItemKind::GoldenPickaxe
            | ItemKind::DiamondPickaxe
            | ItemKind::NetheritePickaxe => 1.2,

            ItemKind::WoodenAxe => 0.8,
            ItemKind::GoldenAxe => 1.0,
            ItemKind::StoneAxe => 0.8,
            ItemKind::IronAxe => 0.9,
            ItemKind::DiamondAxe => 1.0,
            ItemKind::NetheriteAxe => 1.0,

            ItemKind::WoodenHoe => 1.0,
            ItemKind::StoneHoe => 2.0,
            ItemKind::IronHoe => 3.0,
            ItemKind::GoldenHoe => 1.0,
            ItemKind::DiamondHoe => 4.0,
            ItemKind::NetheriteHoe => 4.0,

            ItemKind::Trident => 1.1,

            _ => 4.0,
        }
    }

    fn knockback_resistance(&self) -> f32 {
        match self {
            ItemKind::NetheriteHelmet => 0.1,
            ItemKind::NetheriteChestplate => 0.1,
            ItemKind::NetheriteLeggings => 0.1,
            ItemKind::NetheriteBoots => 0.1,
            _ => 0.0,
        }
    }
}
