use bitflags::bitflags;
use std::collections::HashMap;

enum InteractionType {
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

#[derive(Clone, Hash, Eq, PartialEq)]
enum Intent {
    Air,
    Plant,
    Leaf,
    Growth,
    AbsorbAir,
}

#[derive(Default)]
struct Substance {
    mass: i32,
    values: HashMap<Intent, i32>,
}

fn push_intent(intent: Intent, from: &mut Substance, to: &mut Substance) {
    let amount = from.values.insert(intent.clone(), 0).unwrap_or(0);
    *to.values.entry(intent).or_insert(0) += amount;
}

fn internal_interaction(substance: &mut Substance) {
    let mut becomes = |source: Intent, intent: Intent, percent: i32| {
        if let Some(&val) = substance.values.get(&source) {
            let target = (val * percent) / 100;
            let count = *substance.values.get(&intent).unwrap_or(&0);
            if count < target {
                let delta = std::cmp::max(1, (target - count) / 10);
                *substance.values.entry(intent).or_insert(0) += delta;
                *substance.values.entry(source).or_insert(0) -= delta;
            }
        }
    };
    let merges = |a: Intent, b: Intent, into: Intent| {};

    becomes(Intent::Plant, Intent::Leaf, 5);
    becomes(Intent::Plant, Intent::Growth, 2);
}

// TODO eventually rules for interactions should be categorized and driven by
// categories (for example there will be many interactions that work like plant
// => leaf with a fixed % transformation target).
fn interaction(
    interaction_type: InteractionType,
    a: &mut Substance,
    b: &mut Substance,
    out: Option<&mut Substance>,
) {
    // if interaction_type == Internal {
    //     if ... {

    //     }
    // }
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
        assert_eq!(*s.values.get(&Intent::Leaf).unwrap(), 50);
        // Not 20 because of ordering. I may fix this in the future -- TBD.
        assert_eq!(*s.values.get(&Intent::Growth).unwrap(), 19);

        // Do nothing at limit
        let mut s = Substance::default();
        s.values.insert(Intent::Plant, 1000);
        s.values.insert(Intent::Leaf, 50);
        s.values.insert(Intent::Growth, 20);
        internal_interaction(&mut s);
        assert_eq!(*s.values.get(&Intent::Plant).unwrap(), 1000);
        assert_eq!(*s.values.get(&Intent::Leaf).unwrap(), 50);
        assert_eq!(*s.values.get(&Intent::Growth).unwrap(), 20);

        // Do nothing over limit
        let mut s = Substance::default();
        s.values.insert(Intent::Plant, 100);
        s.values.insert(Intent::Leaf, 100);
        s.values.insert(Intent::Growth, 100);
        internal_interaction(&mut s);
        assert_eq!(*s.values.get(&Intent::Plant).unwrap(), 100);
        assert_eq!(*s.values.get(&Intent::Leaf).unwrap(), 100);
        assert_eq!(*s.values.get(&Intent::Growth).unwrap(), 100);
    }

    #[test]
    fn generates_leaf_and_growth_from_plant() {}

    #[test]
    fn internal_interaction_at_target() {}
}
