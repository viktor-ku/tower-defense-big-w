use std::collections::HashMap;

use bevy::prelude::*;

use crate::components::EnemyKind;
use crate::constants::Tunables;

#[derive(Clone, Copy, Debug)]
pub struct Multipliers {
    pub hp: f32,
    pub dmg: f32,
    pub spd: f32,
}

impl Default for Multipliers {
    fn default() -> Self {
        Self {
            hp: 1.0,
            dmg: 1.0,
            spd: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum StatScale {
    Const(f32),
    Linear { start: f32, per_wave: f32 },
    Exp { factor_per_wave: f32 },
}

impl StatScale {
    /// 1-based waves. Wave 1 uses exponent 0 for Exp and adds 0 for Linear.
    pub fn evaluate(&self, wave: u32) -> f32 {
        match *self {
            StatScale::Const(v) => v,
            StatScale::Linear { start, per_wave } => {
                start + per_wave * ((wave.saturating_sub(1)) as f32)
            }
            StatScale::Exp { factor_per_wave } => {
                factor_per_wave.powi((wave.saturating_sub(1)) as i32)
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct KindRule {
    pub health: StatScale,
    pub damage: StatScale,
    pub speed: StatScale,
}

impl Default for KindRule {
    fn default() -> Self {
        Self {
            health: StatScale::Const(1.0),
            damage: StatScale::Const(1.0),
            speed: StatScale::Const(1.0),
        }
    }
}

impl KindRule {
    pub fn evaluate(&self, wave: u32) -> Multipliers {
        Multipliers {
            hp: self.health.evaluate(wave),
            dmg: self.damage.evaluate(wave),
            spd: self.speed.evaluate(wave),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CountCurve {
    Linear { start: u32, per_wave: u32 },
}

impl CountCurve {
    pub fn evaluate(&self, wave: u32) -> u32 {
        match *self {
            CountCurve::Linear { start, per_wave } => start + per_wave * wave.saturating_sub(1),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Weights(pub HashMap<EnemyKind, f32>);

impl Weights {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn set(mut self, kind: EnemyKind, weight: f32) -> Self {
        self.0.insert(kind, weight);
        self
    }
    pub fn normalized(&self) -> HashMap<EnemyKind, f32> {
        let sum: f32 = self.0.values().copied().sum();
        if sum <= 0.0 {
            return self.0.clone();
        }
        self.0.iter().map(|(k, v)| (*k, v / sum)).collect()
    }
}

#[derive(Clone, Debug, Default)]
pub struct Edit {
    pub boss: Option<bool>,
    pub health_mul: f32,
    pub damage_mul: f32,
    pub speed_mul: f32,
    pub composition: Option<Weights>,
}

impl Edit {
    pub fn identity() -> Self {
        Self {
            boss: None,
            health_mul: 1.0,
            damage_mul: 1.0,
            speed_mul: 1.0,
            composition: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum RuleNode {
    Exact(u32, Edit),
    Range(std::ops::RangeInclusive<u32>, Edit),
    Every(u32, Edit),
    NthBoss(u32, Edit),
    PerKind(EnemyKind, KindRule),
}

#[derive(Resource, Clone, Debug)]
pub struct WaveRules {
    pub count: CountCurve,
    pub global: KindRule,
    pub per_kind: HashMap<EnemyKind, KindRule>,
    pub composition: Weights,
    pub boss_every: Option<u32>,
    pub nodes: Vec<RuleNode>,
}

impl Default for WaveRules {
    fn default() -> Self {
        Self {
            count: CountCurve::Linear {
                start: 10,
                per_wave: 2,
            },
            global: KindRule::default(),
            per_kind: HashMap::new(),
            composition: Weights::new()
                .set(EnemyKind::Minion, 0.6)
                .set(EnemyKind::Zombie, 0.4),
            boss_every: Some(10),
            nodes: Vec::new(),
        }
    }
}

pub struct WaveRulesBuilder {
    rules: WaveRules,
}

impl WaveRulesBuilder {
    pub fn new() -> Self {
        Self {
            rules: WaveRules::default(),
        }
    }
    pub fn defaults_count_linear(mut self, start: u32, per_wave: u32) -> Self {
        self.rules.count = CountCurve::Linear { start, per_wave };
        self
    }
    pub fn defaults_scales(
        mut self,
        health: StatScale,
        damage: StatScale,
        speed: StatScale,
    ) -> Self {
        self.rules.global = KindRule {
            health,
            damage,
            speed,
        };
        self
    }
    pub fn defaults_health(mut self, health: StatScale) -> Self {
        self.rules.global.health = health;
        self
    }
    pub fn defaults_damage(mut self, damage: StatScale) -> Self {
        self.rules.global.damage = damage;
        self
    }
    pub fn defaults_speed(mut self, speed: StatScale) -> Self {
        self.rules.global.speed = speed;
        self
    }
    pub fn defaults_composition(mut self, weights: Weights) -> Self {
        self.rules.composition = weights;
        self
    }
    pub fn defaults_boss_every(mut self, every: Option<u32>) -> Self {
        self.rules.boss_every = every;
        self
    }
    pub fn every(mut self, n: u32, edit: Edit) -> Self {
        self.rules.nodes.push(RuleNode::Every(n, edit));
        self
    }
    pub fn range(mut self, start: u32, end: u32, edit: Edit) -> Self {
        self.rules.nodes.push(RuleNode::Range(start..=end, edit));
        self
    }
    pub fn wave(mut self, n: u32, edit: Edit) -> Self {
        self.rules.nodes.push(RuleNode::Exact(n, edit));
        self
    }
    pub fn nth_boss(mut self, n: u32, edit: Edit) -> Self {
        self.rules.nodes.push(RuleNode::NthBoss(n, edit));
        self
    }
    pub fn per_kind(mut self, kind: EnemyKind, rule: KindRule) -> Self {
        self.rules.per_kind.insert(kind, rule);
        self
    }
    pub fn build(self) -> WaveRules {
        self.rules
    }
}

#[derive(Clone, Debug)]
pub struct WavePlan {
    pub enemies: Vec<EnemyKind>,
    pub multipliers: HashMap<EnemyKind, Multipliers>,
    pub is_boss: bool,
}

impl WaveRules {
    pub fn plan(&self, wave: u32, _tunables: &Tunables, seed: Option<u64>) -> WavePlan {
        let mut is_boss = self
            .boss_every
            .map(|n| n > 0 && wave % n == 0)
            .unwrap_or(false);

        // Accumulate edit multipliers (multiplicative), last-wins for boss/composition
        let mut acc = Edit::identity();

        // per-kind scales
        // precedence: defaults -> per_kind -> every -> range -> wave -> nth_boss
        // apply in that order for boss/composition assignment; multipliers multiply in any order
        // 1) defaults already reflected by global KindRule

        // 2) per_kind rules already stored for later; nothing to change in acc

        // 3) every(N)
        for node in &self.nodes {
            if let RuleNode::Every(n, edit) = node {
                if *n != 0 && wave % *n == 0 {
                    apply_edit(&mut acc, edit.clone());
                }
            }
        }
        // 4) range
        for node in &self.nodes {
            if let RuleNode::Range(range, edit) = node {
                if range.contains(&wave) {
                    apply_edit(&mut acc, edit.clone());
                }
            }
        }
        // 5) wave(N)
        for node in &self.nodes {
            if let RuleNode::Exact(n, edit) = node {
                if *n == wave {
                    apply_edit(&mut acc, edit.clone());
                }
            }
        }
        // 6) boss/nth_boss
        if is_boss {
            if let Some(b) = acc.boss {
                is_boss = b;
            }
            let boss_index = self
                .boss_every
                .map(|n| if n == 0 { 0 } else { wave / n })
                .unwrap_or(0);
            for node in &self.nodes {
                if let RuleNode::NthBoss(n, edit) = node {
                    if *n == boss_index && boss_index > 0 {
                        apply_edit(&mut acc, edit.clone());
                    }
                }
            }
        }

        // Build multipliers per kind: global then per-kind override scales, then acc multipliers applied equally to all kinds
        let mut multipliers: HashMap<EnemyKind, Multipliers> = HashMap::new();
        // enumerate current kinds; keep explicit list to avoid FromIterator missing variants
        let kinds = [EnemyKind::Minion, EnemyKind::Zombie, EnemyKind::Boss];
        for kind in kinds {
            let base = if let Some(rule) = self.per_kind.get(&kind) {
                rule.evaluate(wave)
            } else {
                self.global.evaluate(wave)
            };
            multipliers.insert(
                kind,
                Multipliers {
                    hp: base.hp * acc.health_mul,
                    dmg: base.dmg * acc.damage_mul,
                    spd: base.spd * acc.speed_mul,
                },
            );
        }

        // Determine enemy list via weights
        let count = self.count.evaluate(wave) as usize;
        let mut list: Vec<EnemyKind> = Vec::with_capacity(count + if is_boss { 1 } else { 0 });

        let weights = acc
            .composition
            .as_ref()
            .unwrap_or(&self.composition)
            .normalized();
        let w_minion = *weights.get(&EnemyKind::Minion).unwrap_or(&0.6);
        let w_zombie = *weights.get(&EnemyKind::Zombie).unwrap_or(&0.4);
        let sum = (w_minion + w_zombie).max(0.0001);
        let m = ((w_minion / sum) * count as f32).floor() as usize;
        let z = count.saturating_sub(m);
        for _ in 0..m {
            list.push(EnemyKind::Minion);
        }
        for _ in 0..z {
            list.push(EnemyKind::Zombie);
        }

        // Seeded shuffle for composition randomness
        if let Some(world_seed) = seed {
            use rand::{Rng, SeedableRng, rngs::StdRng};
            let seed = world_seed ^ ((wave as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
            let mut rng = StdRng::seed_from_u64(seed);
            use rand::seq::SliceRandom;
            list.shuffle(&mut rng);
        } else {
            use rand::seq::SliceRandom;
            let mut rng = rand::rng();
            list.shuffle(&mut rng);
        }

        if is_boss {
            list.push(EnemyKind::Boss);
        }

        WavePlan {
            enemies: list,
            multipliers,
            is_boss,
        }
    }
}

fn apply_edit(acc: &mut Edit, new_e: Edit) {
    acc.health_mul *= new_e.health_mul.max(0.0);
    acc.damage_mul *= new_e.damage_mul.max(0.0);
    acc.speed_mul *= new_e.speed_mul.max(0.0);
    if let Some(b) = new_e.boss {
        acc.boss = Some(b);
    }
    if let Some(c) = new_e.composition {
        acc.composition = Some(c);
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct WaveSchedule {
    pub plans: Vec<WavePlan>,
}

impl WaveSchedule {
    pub fn precompute(max_waves: u32, rules: &WaveRules, tunables: &Tunables, seed: u64) -> Self {
        let mut plans = Vec::with_capacity(max_waves as usize);
        for w in 1..=max_waves {
            plans.push(rules.plan(w, tunables, Some(seed)));
        }
        WaveSchedule { plans }
    }
}
