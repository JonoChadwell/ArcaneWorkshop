use std::collections::HashMap;
use std::ops::Index;

#[derive(Clone, Hash, Eq, PartialEq)]
pub enum InteractionType {
    Internal,
    Contact,
    Life,
}

// TODO consider re-adding structured conflict types as shorthand for generating
// the conflict map between Intents. For now this is skipped.
// bitflags! {
//     #[derive(Default)]
//     pub struct Conflicts: u32 {
//         const SUN_VOID = 1 << 0;
//         const TOOL_TARGET = 1 << 1;
//         const HOT_COLD = 1 << 2;
//         const FOO_BAR = 1 << 3;
//         // const STRANGE_CHARM = 1 << 2;
//         // const FIZZ_BUZZ = 1 << 6;
//         // const WORKING_BROKEN = 1 << 3;
//         // const ALIVE_DEAD = 1 << 4;
//         // const BIG_SMALL = 1 << 5;
//         // const SHARP_DULL = 1 << 7;
//         // const LIGHT_DARK = 1 << 8;
//         // const HARD_SOFT = 1 << 9;
//         // const ATTACK_DEFEND = 1 << 10;
//     }
// }

// #[derive(Default)]
// struct Intent {
//     name: &'static str,
//     // supports: Conflicts,
//     // negates: Conflicts,
// }

// static INTENT_DATABASE: &[Intent] = &[
//     // Intent {
//     //     name: "Knife",
//     //     // supports: Conflicts::TOOL_TARGET,
//     //     // negates: Conflicts::empty(),
//     // },
//     Intent {
//         name: "Air",
//     }
//     Intent {
//         name: "Plant",
//         // supports: Conflicts::empty(),
//         // negates: Conflicts::TOOL_TARGET,
//     },
//     Intent {
//         name: "Leaf",
//         // supports: Conflicts::empty(),
//         // negates: Conflicts::TOOL_TARGET,
//     },
//     Intent {
//         name: "Growth",
//         // supports: Conflicts::empty(),
//         // negates: Conflicts::empty(),
//     },
//     Intent {
//         name: "AbsorbAir",
//         // supports: Conflicts::empty(),
//         // negates: Conflicts::empty(),
//     },
// ];

// TODO spirit list:
// Air spirit -- Empty
// LeafPlantStem Spirit
//  - If no leaf (child entity), push intent leaf into air to create one
// LeafPlantLeaf Spirit

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Intent {
    Air,
    Plant,
    Leaf,
    Growth,
    AbsorbAir,
}

#[derive(Default)]
pub struct Substance {
    pub mass: i32,
    values: HashMap<Intent, i32>,
}

impl Index<Intent> for Substance {
    type Output = i32;
    fn index(&self, intent: Intent) -> &Self::Output {
        self.values.get(&intent).unwrap_or(&0)
    }
}

impl Substance {
    pub fn new(mass: i32) -> Self {
        Self {
            mass,
            ..Default::default()
        }
    }

    pub fn with(mut self, intent: Intent, amount: i32) -> Self {
        assert!(!self.values.contains_key(&intent));
        self.values.insert(intent, amount);
        self
    }

    pub fn has(&self, intent: Intent) -> bool {
        self[intent] > 0
    }

    // Adds intent using mass as a soft-cap. The further you go the more input
    // is required to step up.
    pub fn add(&mut self, intent: Intent, amount: i32) {
        let mass = std::cmp::max(1, self.mass);
        assert!(0 <= amount);   
        if amount == 0 {
            return;
        }
        let mut amount = amount;
        let value = self.values.entry(intent).or_insert(0);
        while let bracket = (*value) / mass
            && bracket < amount
        {
            let bracket_space = bracket * mass + mass - *value;
            let to_add = std::cmp::min(amount / (bracket + 1), bracket_space);
            assert!(0 < to_add);
            *value += to_add;
            amount -= to_add * (bracket + 1);
        }
        // remainder is lost
    }

    fn add_flat(&mut self, intent: Intent, amount: i32) {
        assert!(0 <= amount);
        if amount == 0 {
            return;
        }
        *self.values.entry(intent).or_insert(0) += amount;
    }

    pub fn sub(&mut self, intent: Intent, amount: i32) {
        assert!(0 <= amount);
        if amount == 0 {
            return;
        }
        let value = self.values.entry(intent).or_insert(0);
        assert!(amount <= *value);
        *value -= amount;
    }

    pub fn modify(&mut self, intent: Intent, delta: i32) {
        if delta < 0 {
            self.add(intent, delta);
        } else {
            self.sub(intent, -delta);
        }
    }

    pub fn print(&self, name: &str) {
        println!("--- {} ---  mass: {}", name, self.mass);
        for (intent, val) in &self.values {
            println!("  {:?}: {}", intent, val);
        }
    }

    pub fn clear(&mut self, intent: Intent) -> i32 {
        self.values.remove(&intent).unwrap_or(0)
    }

    pub fn push(&mut self, intent: Intent, to: &mut Self) {
        to.add(intent, self.clear(intent));
    }

    // Intentionally private. Go through `with` or `add`.
    fn set(&mut self, intent: Intent, value: i32) {
        *self.values.entry(intent).or_insert(0) = value;
    }
    
    pub fn balance(&mut self, intent: Intent, other: &mut Substance) {
        let total_val = self[intent] + other[intent];
        let total_mass = self.mass + other.mass;
        // Round towards self.
        let other_val = total_val * other.mass / total_mass;
        self.set(intent, total_val - other_val);
        other.set(intent, other_val);
    }

}

pub fn internal_interaction(substance: &mut Substance) {
    let mut becomes = |source: Intent, dest: Intent, target_percent: i32, speed_percent: i32| {
        if let Some(&val) = substance.values.get(&source) {
            let target_val = val * target_percent / 100;
            let dest_val = substance[dest];
            if dest_val < target_val {
                let delta = std::cmp::max(1, (target_val - dest_val) * speed_percent / 100);
                substance.sub(source, delta);
                substance.add_flat(dest, delta);
            }
        }
    };

    becomes(Intent::Plant, Intent::Leaf, 5, 10);
    becomes(Intent::Plant, Intent::Growth, 2, 10);

    let mut merge_one = |a: Intent, b: Intent, into: Intent| {
        if substance.has(a) && substance.has(b) {
            substance.sub(a, 1);
            substance.sub(b, 1);
            substance.add(into, 2);
        }
    };

    merge_one(Intent::Leaf, Intent::Air, Intent::Plant);

    let mut merges = |a: Intent, b: Intent, into: Intent, percent: i32| {
        let target = (std::cmp::min(substance[a], substance[b]) * percent) / 100;
        if substance[into] < target {
            substance.sub(a, 1);
            substance.sub(b, 1);
            substance.add(into, 2);
        }
    };

    merges(Intent::Leaf, Intent::Growth, Intent::AbsorbAir, 90);
}

fn fragment_pure(substance: &Substance, fragment_mass: i32) -> (Substance, Substance) {
    let original_mass = substance.mass;
    assert!(0 < fragment_mass);
    assert!(original_mass > fragment_mass);

    let mut remainder = Substance::default();
    remainder.mass = original_mass - fragment_mass;

    let mut fragment = Substance::default();
    fragment.mass = fragment_mass;

    for (intent, &original) in &substance.values {
        let mut delta = original * fragment_mass / original_mass;
        if original > 2 {
            delta = std::cmp::max(1, delta);
            delta = std::cmp::min(original - 1, delta);
        }
        if delta > 0 {
            fragment.values.insert(intent.clone(), delta);
            let rem = original - delta;
            if rem > 0 {
                remainder.values.insert(intent.clone(), rem);
            }
        } else {
            remainder.values.insert(intent.clone(), original);
        }
    }

    (remainder, fragment)
}

pub fn fragment(substance: &mut Substance, fragment_mass: i32) -> Substance {
    let original_mass = substance.mass;
    assert!(0 < fragment_mass);
    assert!(original_mass > fragment_mass);
    substance.mass -= fragment_mass;

    let mut new_substance = Substance::default();
    new_substance.mass = fragment_mass;

    for (intent, value) in &mut substance.values {
        let original = *value;
        let mut delta = original * fragment_mass / original_mass;
        if original > 2 {
            delta = std::cmp::max(1, delta);
            delta = std::cmp::min(original - 1, delta);
        }
        if delta > 0 {
            new_substance.values.insert(intent.clone(), delta);
            *value -= delta;
        }
    }

    new_substance
}

pub fn substance_add(substance: &mut Substance, add: Substance) {
    substance.mass += add.mass;
    for (intent, value) in add.values {
        *substance.values.entry(intent).or_insert(0) += value;
    }
}

// TODO eventually rules for interactions should be categorized and driven by
// categories (for example there will be many interactions that work like plant
// => leaf with a fixed % transformation target).
pub fn interaction(
    interaction_type: InteractionType,
    a: &mut Substance,
    b: &mut Substance,
    out: Option<&mut Substance>,
) {
    fn min_of_three<T: Ord>(a: T, b: T, c: T) -> T {
        std::cmp::min(std::cmp::min(a, b), c)
    }
    if interaction_type == InteractionType::Contact {
        if let Some(out) = out {
            if has_intent(a, Intent::AbsorbAir) && has_intent(b, Intent::Air) && 1 < b.mass {
                let delta = min_of_three(
                    check_intent(a, Intent::AbsorbAir),
                    check_intent(b, Intent::Air),
                    b.mass - 1,
                );
                modify_intent(a, Intent::AbsorbAir, -delta);
                modify_intent(b, Intent::Air, -delta);
                substance_add(out, fragment(b, delta));
            }
            if has_intent(b, Intent::AbsorbAir) && has_intent(a, Intent::Air) && 1 < a.mass {
                let delta = min_of_three(
                    check_intent(b, Intent::AbsorbAir),
                    check_intent(a, Intent::Air),
                    a.mass - 1,
                );
                modify_intent(b, Intent::AbsorbAir, -delta);
                modify_intent(a, Intent::Air, -delta);
                substance_add(out, fragment(a, delta));
            }
        }
        let a_leaf = check_intent(a, Intent::Leaf);
        let b_leaf = check_intent(b, Intent::Leaf);
        let a_air = check_intent(a, Intent::Air);
        let b_air = check_intent(b, Intent::Air);
        const LEAF_GATHER_PERCENT: i32 = 100;
        if 0 < a_leaf && 0 < b_air {
            let target = std::cmp::min(a_leaf, b_air) * LEAF_GATHER_PERCENT / 100;
            let delta = target - a_air;
            if 0 < delta {
                modify_intent(b, Intent::Air, -delta);
                modify_intent(a, Intent::Air, delta);
            }
        }
        if 0 < b_leaf && 0 < a_air {
            let target = std::cmp::min(b_leaf, a_air) * LEAF_GATHER_PERCENT / 100;
            let delta = target - b_air;
            if 0 < delta {
                modify_intent(a, Intent::Air, -delta);
                modify_intent(b, Intent::Air, delta);
            }
        }
    }
}

// // A tool that may or may not have some foreign substance on it.
// struct Tool {
//     body: Substance,
//     // TODO handle grime in tool/target interactions.
//     grime: Option<Substance>,
// }

// fn record_charge(substance: &mut Substance, charge: Charge) {
//     assert!(charge.0.bits().count_ones() == 1);
//     substance.continuum.values.push(charge);
// }

// fn check_charge(substance: &Substance, against: Conflicts) -> i32 {
//     substance.continuum.values.iter()
//         .filter(|(conflicts, _)| *conflicts & against != Conflicts::empty())
//         .map(|(_, value)| value)
//         .sum()
// }

// fn resolve_conflicts(substance: &mut Substance) -> Option<Substance> {
//     let _ = substance.mass;
//     let _ = substance;
//     None
// }

// // Depending on conflict type this may create a new substance or adjust the
// // target substance.
// fn ToolTargetInteraction(Substance tool, Substance target, Substance output) {

// }

// fn SunlightInteraction(Substance sun, Substance target) {

// }

// fn SubmersedInteraction(Substance fluid, Substance target) {

// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_interaction_test() {
        // Does nothing when empty
        let mut s = Substance::default();
        internal_interaction(&mut s);
        assert_eq!(s.values.get(&Intent::Leaf), None);
        assert_eq!(s.values.get(&Intent::Growth), None);

        // Check generators
        s.values.insert(Intent::Plant, 10000);
        internal_interaction(&mut s);
        assert_eq!(*s.values.get(&Intent::Plant).unwrap(), 9931);
        assert_eq!(*s.values.get(&Intent::Leaf).unwrap(), 49);
        assert_eq!(*s.values.get(&Intent::Growth).unwrap(), 18);
        assert_eq!(*s.values.get(&Intent::AbsorbAir).unwrap(), 1);

        // Do nothing over limit
        let mut s = Substance::default();
        s.values.insert(Intent::Plant, 100);
        s.values.insert(Intent::Leaf, 100);
        s.values.insert(Intent::Growth, 100);
        s.values.insert(Intent::AbsorbAir, 100);
        internal_interaction(&mut s);
        assert_eq!(*s.values.get(&Intent::Plant).unwrap(), 100);
        assert_eq!(*s.values.get(&Intent::Leaf).unwrap(), 100);
        assert_eq!(*s.values.get(&Intent::Growth).unwrap(), 100);
        assert_eq!(*s.values.get(&Intent::AbsorbAir).unwrap(), 100);
    }

    #[test]
    fn interaction_test() {
        let mut a = Substance::default();
        a.values.insert(Intent::AbsorbAir, 10);
        a.mass = 10;
        let mut b = Substance::default();
        b.values.insert(Intent::Air, 10);
        b.mass = 10;
        let mut out = Substance::default();

        interaction(InteractionType::Contact, &mut a, &mut b, Some(&mut out));
        assert_eq!(out.mass, 9);
        assert_eq!(check_intent(&a, Intent::AbsorbAir), 1);
        assert_eq!(check_intent(&b, Intent::Air), 1);

        // No out -> no effect
        let mut a = Substance::default();
        a.values.insert(Intent::AbsorbAir, 10);
        a.mass = 10;
        let mut c = Substance::default();
        c.values.insert(Intent::Air, 10);
        c.mass = 10;
        interaction(InteractionType::Contact, &mut a, &mut c, None);
        assert_eq!(c.mass, 10);

        // Wrong type -> no effect
        let mut d = Substance::default();
        d.values.insert(Intent::AbsorbAir, 10);
        d.mass = 10;
        let mut e = Substance::default();
        e.values.insert(Intent::Air, 10);
        e.mass = 10;
        interaction(
            InteractionType::Internal,
            &mut d,
            &mut e,
            Some(&mut Substance::default()),
        );
        assert_eq!(d.mass, 10);
        assert_eq!(e.mass, 10);
    }

    #[test]
    fn substance_test() {
        let mut a = Substance::new(10)
            .with(Intent::Leaf, 5)
            .with(Intent::Air, 25);
        a.add(Intent::Leaf, 5);
        assert_eq!(a[Intent::Leaf], 10);
        a.add(Intent::Air, 15);
        assert_eq!(a[Intent::Air], 30);
        a.add(Intent::Air, 5);
        assert_eq!(a[Intent::Air], 31);
        a.add(Intent::Air, (4 * 9) + (5 * 5) + 3);
        assert_eq!(a[Intent::Air], 45);
    }
}
