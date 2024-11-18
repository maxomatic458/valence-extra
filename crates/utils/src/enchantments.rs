use std::collections::HashMap;

use valence::{
    nbt::{value::ValueRef, Value},
    ItemStack,
};

// https://help.minecraft.net/hc/en-us/articles/360058730912-Minecraft-List-of-Enchantments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Enchantment {
    AquaAffinity,
    BlastProtection,
    CurseOfBinding,
    DepthStrider,
    FeatherFalling,
    FireProtection,
    FrostWalker,
    ProjectileProtection,
    Protection,
    Respiration,
    SoulSpeed,
    Thorns,
    SwiftSneak,
    BaneOfArthropods,
    // Breach,
    // Density,
    Efficiency,
    FireAspect,
    Looting,
    Impaling,
    Knockback,
    Sharpness,
    Smite,
    SweepingEdge,
    Channeling,
    Flame,
    Infinity,
    Loyalty,
    Riptide,
    Multishot,
    Piercing,
    Power,
    Punch,
    QuickCharge,
    Fortune,
    LuckOftheSea,
    Lure,
    SilkTouch,
}

impl Enchantment {
    pub fn id(&self) -> &'static str {
        match self {
            Enchantment::AquaAffinity => "aqua_affinity",
            Enchantment::BlastProtection => "blast_protection",
            Enchantment::CurseOfBinding => "binding_curse",
            Enchantment::DepthStrider => "depth_strider",
            Enchantment::FeatherFalling => "feather_falling",
            Enchantment::FireProtection => "fire_protection",
            Enchantment::FrostWalker => "frost_walker",
            Enchantment::ProjectileProtection => "projectile_protection",
            Enchantment::Protection => "protection",
            Enchantment::Respiration => "respiration",
            Enchantment::SoulSpeed => "soul_speed",
            Enchantment::Thorns => "thorns",
            Enchantment::SwiftSneak => "swift_sneak",
            Enchantment::BaneOfArthropods => "bane_of_arthropods",
            Enchantment::Efficiency => "efficiency",
            Enchantment::FireAspect => "fire_aspect",
            Enchantment::Looting => "looting",
            Enchantment::Impaling => "impaling",
            Enchantment::Knockback => "knockback",
            Enchantment::Sharpness => "sharpness",
            Enchantment::Smite => "smite",
            Enchantment::SweepingEdge => "sweeping",
            Enchantment::Channeling => "channeling",
            Enchantment::Flame => "flame",
            Enchantment::Infinity => "infinity",
            Enchantment::Loyalty => "loyalty",
            Enchantment::Riptide => "riptide",
            Enchantment::Multishot => "multishot",
            Enchantment::Piercing => "piercing",
            Enchantment::Power => "power",
            Enchantment::Punch => "punch",
            Enchantment::QuickCharge => "quick_charge",
            Enchantment::Fortune => "fortune",
            Enchantment::LuckOftheSea => "luck_of_the_sea",
            Enchantment::Lure => "lure",
            Enchantment::SilkTouch => "silk_touch",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "aqua_affinity" | "minecraft:aqua_affinity" => Some(Enchantment::AquaAffinity),
            "blast_protection" | "minecraft:blast_protection" => Some(Enchantment::BlastProtection),
            "binding_curse" | "minecraft:binding_curse" => Some(Enchantment::CurseOfBinding),
            "depth_strider" | "minecraft:depth_strider" => Some(Enchantment::DepthStrider),
            "feather_falling" | "minecraft:feather_falling" => Some(Enchantment::FeatherFalling),
            "fire_protection" | "minecraft:fire_protection" => Some(Enchantment::FireProtection),
            "frost_walker" | "minecraft:frost_walker" => Some(Enchantment::FrostWalker),
            "projectile_protection" | "minecraft:projectile_protection" => {
                Some(Enchantment::ProjectileProtection)
            }
            "protection" | "minecraft:protection" => Some(Enchantment::Protection),
            "respiration" | "minecraft:respiration" => Some(Enchantment::Respiration),
            "soul_speed" | "minecraft:soul_speed" => Some(Enchantment::SoulSpeed),
            "thorns" | "minecraft:thorns" => Some(Enchantment::Thorns),
            "swift_sneak" | "minecraft:swift_sneak" => Some(Enchantment::SwiftSneak),
            "bane_of_arthropods" | "minecraft:bane_of_arthropods" => {
                Some(Enchantment::BaneOfArthropods)
            }
            "efficiency" | "minecraft:efficiency" => Some(Enchantment::Efficiency),
            "fire_aspect" | "minecraft:fire_aspect" => Some(Enchantment::FireAspect),
            "looting" | "minecraft:looting" => Some(Enchantment::Looting),
            "impaling" | "minecraft:impaling" => Some(Enchantment::Impaling),
            "knockback" | "minecraft:knockback" => Some(Enchantment::Knockback),
            "sharpness" | "minecraft:sharpness" => Some(Enchantment::Sharpness),
            "smite" | "minecraft:smite" => Some(Enchantment::Smite),
            "sweeping" | "minecraft:sweeping" => Some(Enchantment::SweepingEdge),
            "channeling" | "minecraft:channeling" => Some(Enchantment::Channeling),
            "flame" | "minecraft:flame" => Some(Enchantment::Flame),
            "infinity" | "minecraft:infinity" => Some(Enchantment::Infinity),
            "loyalty" | "minecraft:loyalty" => Some(Enchantment::Loyalty),
            "riptide" | "minecraft:riptide" => Some(Enchantment::Riptide),
            "multishot" | "minecraft:multishot" => Some(Enchantment::Multishot),
            "piercing" | "minecraft:piercing" => Some(Enchantment::Piercing),
            "power" | "minecraft:power" => Some(Enchantment::Power),
            "punch" | "minecraft:punch" => Some(Enchantment::Punch),
            "quick_charge" | "minecraft:quick_charge" => Some(Enchantment::QuickCharge),
            "fortune" | "minecraft:fortune" => Some(Enchantment::Fortune),
            "luck_of_the_sea" | "minecraft:luck_of_the_sea" => Some(Enchantment::LuckOftheSea),
            "lure" | "minecraft:lure" => Some(Enchantment::Lure),
            "silk_touch" | "minecraft:silk_touch" => Some(Enchantment::SilkTouch),
            _ => None,
        }
    }
}

pub trait ItemStackEnchantmentsExt {
    fn enchantments(&self) -> HashMap<Enchantment, u32>;
}

impl ItemStackEnchantmentsExt for ItemStack {
    fn enchantments(&self) -> HashMap<Enchantment, u32> {
        let mut enchantments = HashMap::new();
        if let Some(nbt) = &self.nbt {
            if let Some(Value::List(enchants)) = nbt.get("Enchantments") {
                for enchant in enchants {
                    if let ValueRef::Compound(enchant) = enchant {
                        if let (Some(Value::String(id)), Some(Value::Long(level))) =
                            (enchant.get("id"), enchant.get("lvl"))
                        {
                            if let Some(enchantment) = Enchantment::from_id(id) {
                                enchantments.insert(enchantment, *level as u32);
                            }
                        }
                    }
                }
            }
        }

        enchantments
    }
}
