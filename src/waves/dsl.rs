//! Macro-based DSL for declaring wave rules.
//!
//! This module exposes two macros:
//! - `wave_rules!` — declare the full ruleset (defaults, periodic, ranges, per-wave, per-kind)
//! - `wave!` — sugar for a single per-wave edit (returns a `WaveRules` with only that rule)
//!
//! Basic usage
//!
//! ```rust
//! // Build a ruleset with defaults and a periodic boss every 10th wave
//! let rules: td::waves::rules::WaveRules = td::wave_rules! {
//!   defaults {
//!     // enemy count = start + per_wave * (wave-1)
//!     count = linear(10, + 2);
//!     // per-wave multiplicative health scaling (wave 1 => x1.0)
//!     health = exp(1.05);
//!     // damage grows linearly as a multiplier (wave 1 => x1.0)
//!     damage = linear(1.0, + 0.02);
//!     // baseline speed multiplier
//!     speed  = const(1.0);
//!     // composition weights; normalized per wave
//!     composition = weights { td::components::EnemyKind::Minion: 0.6, td::components::EnemyKind::Zombie: 0.4 };
//!     // boss cadence (every Nth wave)
//!     boss_every = 10;
//!   }
//!
//!   // Every 10th wave is a boss wave; you can also tweak stats here
//!   every(10) { boss; }
//! };
//!
//! // Evaluate a plan for a given wave
//! let tunables = td::constants::Tunables::default();
//! let plan = rules.plan(10, &tunables, Some(123));
//! assert!(plan.enemies.len() >= tunables.wave_base_enemy_count as usize);
//! assert!(plan.enemies.iter().any(|k| matches!(k, td::components::EnemyKind::Boss)));
//! ```
//!
//! Per-wave and range overrides
//!
//! ```rust
//! let rules: td::waves::rules::WaveRules = td::wave_rules! {
//!   defaults { count = linear(8, + 1); health = exp(1.02); speed = const(1.0); }
//!   // Make waves 11..=20 hit a little harder overall
//!   range(11..=20) { damage *= 1.1; }
//!   // Make wave 17 fast and zombie-heavy
//!   wave(17) {
//!     speed *= 1.8;
//!     composition = weights { td::components::EnemyKind::Minion: 0.2, td::components::EnemyKind::Zombie: 0.8 };
//!   }
//! };
//! let tunables = td::constants::Tunables::default();
//! let plan = rules.plan(17, &tunables, Some(7));
//! assert!(plan.enemies.iter().any(|k| matches!(k, td::components::EnemyKind::Zombie)));
//! ```
//!
//! Per-kind scaling defaults
//!
//! ```rust
//! let rules: td::waves::rules::WaveRules = td::wave_rules! {
//!   defaults { count = linear(6, + 1); health = exp(1.0); }
//!   // Give Zombies a steeper health curve globally
//!   per_kind(td::components::EnemyKind::Zombie) {
//!     health = exp(1.07);
//!     damage = linear(1.0, + 0.01);
//!   }
//! };
//! let tunables = td::constants::Tunables::default();
//! let p5 = rules.plan(5, &tunables, Some(5));
//! let mul = p5.multipliers.get(&td::components::EnemyKind::Zombie).unwrap();
//! assert!(mul.hp > 1.0);
//! ```
//!
//! Single-wave sugar
//!
//! ```rust
//! // Creates a ruleset where only wave 17 damage is multiplied by 1.13
//! let rules: td::waves::rules::WaveRules = td::wave!(17, it => { it.damage *= 1.13; });
//! let tunables = td::constants::Tunables::default();
//! let p16 = rules.plan(16, &tunables, Some(1));
//! let p17 = rules.plan(17, &tunables, Some(1));
//! let z = td::components::EnemyKind::Zombie;
//! assert!(p17.multipliers.get(&z).unwrap().dmg > p16.multipliers.get(&z).unwrap().dmg);
//! ```
use crate::components::EnemyKind;
use crate::waves::rules::{Edit, KindRule, StatScale, WaveRules, WaveRulesBuilder, Weights};

#[macro_export]
macro_rules! wave_rules {
    // Entry with body using braces directly
    { $($body:tt)* } => {{
        $crate::__wave_rules_expand!{ $($body)* }
    }};
    // Empty
    () => { $crate::waves::rules::WaveRules::default() };
}

#[macro_export]
macro_rules! wave {
    ($n:expr, $it:ident => { $($stmts:tt)* } ) => {{
        let mut $it = $crate::waves::rules::Edit::identity();
        $crate::__edit_stmt!($it; $($stmts)*);
        $crate::waves::rules::WaveRulesBuilder::new().wave($n, $it).build()
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __weights_internal {
    // weights { Kind: val, ... }
    ( $( $kind:path : $val:expr ),* $(,)? ) => {{
        let mut w = $crate::waves::rules::Weights::new();
        $( w = w.set($kind, $val); )*
        w
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __edit_stmt {
    ($acc:ident; ) => {};
    ($acc:ident; boss; $($rest:tt)*) => {{ $acc.boss = Some(true); $crate::__edit_stmt!($acc; $($rest)*); }};
    // Support `it.health *= ...;` style by ignoring the leading ident
    ($acc:ident; $_any:ident . boss; $($rest:tt)*) => {{ $acc.boss = Some(true); $crate::__edit_stmt!($acc; $($rest)*); }};
    ($acc:ident; health *= $v:expr; $($rest:tt)*) => {{ $acc.health_mul *= $v; $crate::__edit_stmt!($acc; $($rest)*); }};
    ($acc:ident; $_any:ident . health *= $v:expr; $($rest:tt)*) => {{ $acc.health_mul *= $v; $crate::__edit_stmt!($acc; $($rest)*); }};
    ($acc:ident; damage *= $v:expr; $($rest:tt)*) => {{ $acc.damage_mul *= $v; $crate::__edit_stmt!($acc; $($rest)*); }};
    ($acc:ident; $_any:ident . damage *= $v:expr; $($rest:tt)*) => {{ $acc.damage_mul *= $v; $crate::__edit_stmt!($acc; $($rest)*); }};
    ($acc:ident; speed *= $v:expr; $($rest:tt)*) => {{ $acc.speed_mul *= $v; $crate::__edit_stmt!($acc; $($rest)*); }};
    ($acc:ident; $_any:ident . speed *= $v:expr; $($rest:tt)*) => {{ $acc.speed_mul *= $v; $crate::__edit_stmt!($acc; $($rest)*); }};
    ($acc:ident; composition = weights { $($w:tt)* } ; $($rest:tt)*) => {{
        let w = $crate::__weights_internal!( $($w)* );
        $acc.composition = Some(w);
        $crate::__edit_stmt!($acc; $($rest)*);
    }};
    ($acc:ident; $_any:ident . composition = weights { $($w:tt)* } ; $($rest:tt)*) => {{
        let w = $crate::__weights_internal!( $($w)* );
        $acc.composition = Some(w);
        $crate::__edit_stmt!($acc; $($rest)*);
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __edit_block {
    ($acc:ident ; { $($stmts:tt)* }) => {{ $crate::__edit_stmt!($acc; $($stmts)* ); }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __per_kind_block {
    // per_kind(Kind) { health = exp(x); damage = linear(a, +b); speed = const(s); }
    ($builder:ident ; $kind:path { $($body:tt)* }) => {{
        let mut health = $crate::waves::rules::StatScale::Const(1.0);
        let mut damage = $crate::waves::rules::StatScale::Const(1.0);
        let mut speed  = $crate::waves::rules::StatScale::Const(1.0);
        $crate::__per_kind_kv!(health, damage, speed; $($body)*);
        $builder = $builder.per_kind($kind, $crate::waves::rules::KindRule{ health, damage, speed });
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __per_kind_kv {
    ($h:ident, $d:ident, $s:ident; ) => {};
    ($h:ident, $d:ident, $s:ident; health = exp($f:expr); $($rest:tt)*) => {{ $h = $crate::waves::rules::StatScale::Exp{ factor_per_wave: $f }; $crate::__per_kind_kv!($h,$d,$s; $($rest)*); }};
    ($h:ident, $d:ident, $s:ident; damage = exp($f:expr); $($rest:tt)*) => {{ $d = $crate::waves::rules::StatScale::Exp{ factor_per_wave: $f }; $crate::__per_kind_kv!($h,$d,$s; $($rest)*); }};
    ($h:ident, $d:ident, $s:ident; speed = exp($f:expr);  $($rest:tt)*) => {{ $s = $crate::waves::rules::StatScale::Exp{ factor_per_wave: $f }; $crate::__per_kind_kv!($h,$d,$s; $($rest)*); }};
    ($h:ident, $d:ident, $s:ident; health = linear($st:expr, + $pw:expr); $($rest:tt)*) => {{ $h = $crate::waves::rules::StatScale::Linear{ start: $st, per_wave: $pw }; $crate::__per_kind_kv!($h,$d,$s; $($rest)*); }};
    ($h:ident, $d:ident, $s:ident; damage = linear($st:expr, + $pw:expr); $($rest:tt)*) => {{ $d = $crate::waves::rules::StatScale::Linear{ start: $st, per_wave: $pw }; $crate::__per_kind_kv!($h,$d,$s; $($rest)*); }};
    ($h:ident, $d:ident, $s:ident; speed = linear($st:expr, + $pw:expr);  $($rest:tt)*) => {{ $s = $crate::waves::rules::StatScale::Linear{ start: $st, per_wave: $pw }; $crate::__per_kind_kv!($h,$d,$s; $($rest)*); }};
    ($h:ident, $d:ident, $s:ident; health = const($c:expr); $($rest:tt)*) => {{ $h = $crate::waves::rules::StatScale::Const($c); $crate::__per_kind_kv!($h,$d,$s; $($rest)*); }};
    ($h:ident, $d:ident, $s:ident; damage = const($c:expr); $($rest:tt)*) => {{ $d = $crate::waves::rules::StatScale::Const($c); $crate::__per_kind_kv!($h,$d,$s; $($rest)*); }};
    ($h:ident, $d:ident, $s:ident; speed = const($c:expr);  $($rest:tt)*) => {{ $s = $crate::waves::rules::StatScale::Const($c); $crate::__per_kind_kv!($h,$d,$s; $($rest)*); }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __wave_rules_expand {
    ($($body:tt)*) => {{
        let mut __b = $crate::waves::rules::WaveRulesBuilder::new();
        $crate::__wave_rules_kvs!(__b; $($body)*);
        __b.build()
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __wave_rules_kvs {
    ($b:ident; ) => {};
    // defaults block
    ($b:ident; defaults { $($dbody:tt)* } $($rest:tt)*) => {{
        $crate::__defaults_block!($b; $($dbody)*);
        $crate::__wave_rules_kvs!($b; $($rest)*);
    }};
    // every(n) { ... }
    ($b:ident; every($n:expr) { $($ebody:tt)* } $($rest:tt)*) => {{
        let mut __e = $crate::waves::rules::Edit::identity();
        $crate::__edit_stmt!(__e; $($ebody)*);
        $b = $b.every($n, __e);
        $crate::__wave_rules_kvs!($b; $($rest)*);
    }};
    // range(a..=b) { ... }
    ($b:ident; range($s:literal ..= $e:literal) { $($rbody:tt)* } $($rest:tt)*) => {{
        let mut __e = $crate::waves::rules::Edit::identity();
        $crate::__edit_stmt!(__e; $($rbody)*);
        $b = $b.range($s as u32, $e as u32, __e);
        $crate::__wave_rules_kvs!($b; $($rest)*);
    }};
    // wave(n) { ... }
    ($b:ident; wave($n:expr) { $($wbody:tt)* } $($rest:tt)*) => {{
        let mut __e = $crate::waves::rules::Edit::identity();
        $crate::__edit_stmt!(__e; $($wbody)*);
        $b = $b.wave($n, __e);
        $crate::__wave_rules_kvs!($b; $($rest)*);
    }};
    // nth_boss(n) { ... }
    ($b:ident; nth_boss($n:expr) { $($nbbody:tt)* } $($rest:tt)*) => {{
        let mut __e = $crate::waves::rules::Edit::identity();
        $crate::__edit_stmt!(__e; $($nbbody)*);
        $b = $b.nth_boss($n, __e);
        $crate::__wave_rules_kvs!($b; $($rest)*);
    }};
    // per_kind(Kind) { ... }
    ($b:ident; per_kind($k:path) { $($pkbody:tt)* } $($rest:tt)*) => {{
        $crate::__per_kind_block!($b; $k { $($pkbody)* });
        $crate::__wave_rules_kvs!($b; $($rest)*);
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! __defaults_block {
    ($b:ident; ) => {};
    ($b:ident; count = linear($st:expr, + $pw:expr); $($rest:tt)*) => {{
        $b = $b.defaults_count_linear($st as u32, $pw as u32);
        $crate::__defaults_block!($b; $($rest)*);
    }};
    ($b:ident; health = exp($f:expr); $($rest:tt)*) => {{
        let h = $crate::waves::rules::StatScale::Exp{ factor_per_wave: $f };
        $b = $b.defaults_health(h);
        $crate::__defaults_block!($b; $($rest)*);
    }};
    ($b:ident; damage = linear($st:expr, + $pw:expr); $($rest:tt)*) => {{
        let d = $crate::waves::rules::StatScale::Linear{ start: $st, per_wave: $pw };
        $b = $b.defaults_damage(d);
        $crate::__defaults_block!($b; $($rest)*);
    }};
    ($b:ident; speed = const($c:expr); $($rest:tt)*) => {{
        let s = $crate::waves::rules::StatScale::Const($c);
        $b = $b.defaults_speed(s);
        $crate::__defaults_block!($b; $($rest)*);
    }};
    ($b:ident; composition = weights { $($w:tt)* }; $($rest:tt)*) => {{
        let w = $crate::__weights_internal!( $($w)* );
        $b = $b.defaults_composition(w);
        $crate::__defaults_block!($b; $($rest)*);
    }};
    ($b:ident; boss_every = $n:expr; $($rest:tt)*) => {{
        $b = $b.defaults_boss_every(Some($n));
        $crate::__defaults_block!($b; $($rest)*);
    }};
}
