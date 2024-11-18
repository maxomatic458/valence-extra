use std::time::{Duration, Instant};

use valence::math::Vec3;

/// Calculates the damage after armor (this is the java edition formula).
/// (java behavior)
pub fn damage_after_armor(damage: f32, armor_points: f32, toughness: f32) -> f32 {
    // https://minecraft.fandom.com/wiki/Armor
    let max_part = (armor_points / 5.0).max(armor_points - (4.0 * damage / (toughness + 8.0)));
    let min_part = max_part.min(20.0);

    let damage_multiplier = 1.0 - (min_part / 25.0);
    damage * damage_multiplier
}

/// Calculates a damage multiplier based on the attack cooldown.
/// (java behavior)
pub fn attack_cooldown_base_damage(weapon_attack_speed: f32, last_attack: Instant) -> f32 {
    // https://minecraft.fandom.com/wiki/Damage
    let elapsed_millis = last_attack.elapsed().as_millis();

    let elapsed_ticks = elapsed_millis as f32 / 50.0;
    let t = 20.0 / weapon_attack_speed;

    0.2 + ((t + 0.5) / elapsed_ticks).powf(2.0) * 0.8
}

/// Calculates a damage multiplier based on the attack cooldown for damage caused by enchantments.
/// (java behavior)
pub fn attack_cooldown_enchantment_damage(weapon_attack_speed: f32, last_attack: Instant) -> f32 {
    // https://minecraft.fandom.com/wiki/Damage
    let elapsed_millis = last_attack.elapsed().as_millis();

    let elapsed_ticks = elapsed_millis as f32 / 50.0;
    let t = 20.0 / weapon_attack_speed;

    0.2 + ((t + 0.5) / elapsed_ticks) * 0.8
}

/// Calculates the damage for the sharpness enchantment.
/// (java behavior)
pub fn enchant_sharpness_damage(damage: f32, level: u32) -> f32 {
    // https://minecraft.fandom.com/wiki/Sharpness

    if level == 0 {
        return damage;
    }

    damage + 0.5 * level as f32 + 0.5
}

/// Calculates the damage for the power enchantment.
// (java behavior)
pub fn enchant_power_damage(damage: f32, level: u32) -> f32 {
    // https://minecraft.fandom.com/wiki/Power

    if level == 0 {
        return damage;
    }

    (damage + 0.25 * (level as f32 + 1.0)).ceil()
}

/// Calculates knockback based on the knockback enchantment level.
/// (maybe java behavior?)
pub fn enchant_knockback(base_knockback: Vec3, level: u32) -> Vec3 {
    // https://minecraft.fandom.com/wiki/Knockback
    // TODO: figure out how java actually does this
    let multiplier = match level {
        0 => 1.0,
        1 => 1.05,
        _ => 1.9,
    };

    base_knockback * multiplier
}

/// Calculates the knockback based on punch enchantment level.
/// (java behavior)
pub fn enchant_punch(base_knockback: Vec3, level: u32) -> Vec3 {
    // https://minecraft.fandom.com/wiki/Punch

    if level == 0 {
        return base_knockback;
    }

    base_knockback + base_knockback.normalize() * (level as f32 + 1.0)
}

/// Calculates the fire aspect burn time and damage per second.
/// (mostly java behavior)
pub fn enchant_fire_aspect(level: u32) -> (Duration, f32) {
    // https://minecraft.fandom.com/wiki/Fire_Aspect
    // in java the first fire damage is not applied
    // so the received damage would be level - 1 over level * 4 seconds

    if level == 0 {
        return (Duration::from_secs(0), 0.0);
    }

    let burn_time = Duration::from_secs(4 * level as u64);
    let damage_per_second = 1.0;

    (burn_time, damage_per_second)
}

/// Calculates the flame burn time and damage per second.
/// (mostly java behavior)
pub fn enchant_flame(level: u32) -> (Duration, f32) {
    // https://minecraft.fandom.com/wiki/Flame
    if level == 0 {
        return (Duration::from_secs(0), 0.0);
    }

    let burn_time = Duration::from_secs(5 * level as u64);
    let damage_per_second = 1.0;

    (burn_time, damage_per_second)
}
