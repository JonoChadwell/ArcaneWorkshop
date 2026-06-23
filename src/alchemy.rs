use bitflags::bitflags;

// Types of conflicts within the alchemy system. Specific conflicts are only
// allowed in specific contexts. TBD which and where.
bitflags! {
    #[derive(Default)]
    pub struct Conflicts: u32 {
        const SUN_VOID = 1 << 0;
        const TOOL_TARGET = 1 << 1;
        const HOT_COLD = 1 << 2;
        const FOO_BAR = 1 << 3;
        // const STRANGE_CHARM = 1 << 2;
        // const FIZZ_BUZZ = 1 << 6;
        // const WORKING_BROKEN = 1 << 3;
        // const ALIVE_DEAD = 1 << 4;
        // const BIG_SMALL = 1 << 5;
        // const SHARP_DULL = 1 << 7;
        // const LIGHT_DARK = 1 << 8;
        // const HARD_SOFT = 1 << 9;
        // const ATTACK_DEFEND = 1 << 10;
    }
}

// #[derive(Default)]
// struct Intent {
//     name: &'static str,
//     supports: Conflicts,
//     negates: Conflicts,
// }

// static INTENT_DATABASE: &[Intent] = &[
//     Intent {
//         name: "Knife",
//         supports: Conflicts::TOOL_TARGET,
//         negates: Conflicts::empty(),
//     },
//     Intent {
//         name: "Plant",
//         supports: Conflicts::empty(),
//         negates: Conflicts::TOOL_TARGET,
//     },
//     Intent {
//         name: "Tool",
//         supports: Conflicts::empty(),
//         negates: Conflicts::empty(),
//     },
// ];

type Charge = (Conflicts, i32);

#[derive(Default)]
struct Continuum {
    values: Vec<Charge>,
    // intents: Vec<&'static Intent>,
}

#[derive(Default)]
struct Substance {
    continuum: Continuum,
    mass: f32,
}

// // A tool that may or may not have some foreign substance on it.
// struct Tool {
//     body: Substance,
//     // TODO handle grime in tool/target interactions.
//     grime: Option<Substance>,
// }

fn record_charge(substance: &mut Substance, charge: Charge) {
    assert!(charge.0.bits().count_ones() == 1);
    substance.continuum.values.push(charge);
}

fn check_charge(substance: &Substance, against: Conflicts) -> i32 {
    substance.continuum.values.iter()
        .filter(|(conflicts, _)| *conflicts & against != Conflicts::empty())
        .map(|(_, value)| value)
        .sum()
}

fn resolve_conflicts(substance: &mut Substance) -> Option<Substance> {
    let _ = substance.mass;
    let _ = substance;
    None
}

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
    fn basic_flags_combination() {
        let both = Conflicts::HOT_COLD | Conflicts::FOO_BAR;
        assert!(both.contains(Conflicts::HOT_COLD));
        assert!(both.contains(Conflicts::FOO_BAR));
    }

    #[test]
    fn charge_manipulation() {
        let mut substance = Substance::default();
        record_charge(&mut substance, (Conflicts::SUN_VOID, 5));
        assert!(check_charge(&substance, Conflicts::SUN_VOID) == 5);
        record_charge(&mut substance, (Conflicts::SUN_VOID, -3));
        assert!(check_charge(&substance, Conflicts::SUN_VOID) == 2);
    }
}
