//! The geologist's kit: water bottle, pickaxe, food and the backpack that
//! holds them. Pure data and rules (no rendering); `game.rs` owns the state.

// ---- water -----------------------------------------------------------------
pub const WATER_MAX: f32 = 100.0;
/// Bottle drain per tick while out in the field and standing still.
pub const WATER_DRAIN_IDLE: f32 = 0.0012;
/// Extra drain per tick when walking at full speed on the flat (scaled by
/// distance moved and by the slope effort).
pub const WATER_DRAIN_WALK: f32 = 0.0045;
/// How close to a lake or river you must be to fill the bottle.
pub const WATER_REACH: f32 = 80.0;
/// Dehydrated (empty bottle): slower and more tiring.
pub const DEHYDRATED_SPEED: f32 = 0.8;
pub const DEHYDRATED_EFFORT: f32 = 1.5;

// ---- food & backpack -------------------------------------------------------
/// Slots shared by food and bonus rocks. The water bottle has its own holder.
pub const BACKPACK_SLOTS: u32 = 12;
pub const APPLE_ENERGY: f32 = 20.0;
pub const SANDWICH_ENERGY: f32 = 40.0;
pub const APPLE_PRICE: u32 = 15;
pub const SANDWICH_PRICE: u32 = 30;
/// You can only eat in the shade: within this distance of a tree.
pub const TREE_REACH: f32 = 110.0;
/// Not hungry above this much energy.
pub const NOT_HUNGRY_ABOVE: f32 = 92.0;

// ---- pickaxe ---------------------------------------------------------------
pub const PICK_REPLACE_COST: u32 = 25;

pub struct PickTier {
    pub name: &'static str,
    pub durability: u32,
    /// Points to upgrade *to* this tier (0 for the starting pickaxe).
    pub cost: u32,
}

pub const PICK_TIERS: [PickTier; 4] = [
    PickTier { name: "Basic", durability: 15, cost: 0 },
    PickTier { name: "Reinforced", durability: 28, cost: 60 },
    PickTier { name: "Hardened", durability: 45, cost: 140 },
    PickTier { name: "Tungsten", durability: 70, cost: 260 },
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pickaxe {
    pub tier: usize,
    pub durability: u32,
}

impl Pickaxe {
    pub fn new(tier: usize) -> Self {
        let tier = tier.min(PICK_TIERS.len() - 1);
        Pickaxe { tier, durability: PICK_TIERS[tier].durability }
    }
    pub fn max_durability(&self) -> u32 {
        PICK_TIERS[self.tier].durability
    }
    pub fn name(&self) -> &'static str {
        PICK_TIERS[self.tier].name
    }
    pub fn broken(&self) -> bool {
        self.durability == 0
    }
    /// Cost and name of the next tier, if there is one.
    pub fn next_tier(&self) -> Option<(&'static str, u32)> {
        PICK_TIERS.get(self.tier + 1).map(|t| (t.name, t.cost))
    }
    /// Apply wear; returns true if this use broke the pickaxe.
    pub fn wear(&mut self, amount: u32) -> bool {
        let was_ok = self.durability > 0;
        self.durability = self.durability.saturating_sub(amount);
        was_ok && self.durability == 0
    }
}

/// How hard a rock is on the pickaxe: crystalline rocks bite harder.
pub fn rock_hardness(lithology: &str) -> u32 {
    match lithology {
        "Igneous" | "Metamorphic" => 2,
        _ => 1,
    }
}

// ---- backpack menu ---------------------------------------------------------
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    RefillWater,
    ReplacePickaxe,
    UpgradePickaxe,
    EatApple,
    EatSandwich,
    BuyApple,
    BuySandwich,
}

pub const ACTIONS: [Action; 7] = [
    Action::RefillWater,
    Action::ReplacePickaxe,
    Action::UpgradePickaxe,
    Action::EatApple,
    Action::EatSandwich,
    Action::BuyApple,
    Action::BuySandwich,
];

#[derive(Clone, Debug)]
pub struct PackRow {
    pub action: Action,
    pub label: String,
    pub detail: String,
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pickaxe_wears_and_breaks_exactly_once() {
        let mut p = Pickaxe::new(0);
        assert_eq!((p.durability, p.max_durability(), p.broken()), (15, 15, false));
        for _ in 0..14 {
            assert!(!p.wear(1));
        }
        assert!(p.wear(1), "the last use breaks it");
        assert!(p.broken());
        assert!(!p.wear(1), "already broken: no second break event");
        assert_eq!(p.durability, 0);
    }

    #[test]
    fn heavy_wear_cannot_underflow() {
        let mut p = Pickaxe::new(0);
        p.durability = 1;
        assert!(p.wear(2));
        assert_eq!(p.durability, 0);
    }

    #[test]
    fn tiers_get_tougher_and_costlier() {
        for w in PICK_TIERS.windows(2) {
            assert!(w[1].durability > w[0].durability && w[1].cost > w[0].cost);
        }
        let p = Pickaxe::new(PICK_TIERS.len() - 1);
        assert!(p.next_tier().is_none());
        assert_eq!(Pickaxe::new(99).tier, PICK_TIERS.len() - 1);
        assert_eq!(Pickaxe::new(0).next_tier(), Some(("Reinforced", 60)));
    }

    #[test]
    fn crystalline_rocks_are_harder_on_tools() {
        assert!(rock_hardness("Igneous") > rock_hardness("Sedimentary"));
        assert_eq!(rock_hardness("Metamorphic"), 2);
    }
}
