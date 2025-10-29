use td::components::enemies::EnemyKind;
use td::components::towers::TowerKind;

#[test]
fn enemy_kind_stats_are_expected() {
    assert_eq!(EnemyKind::Minion.stats(), (30, 5, 24.0, 0.8));
    assert_eq!(EnemyKind::Zombie.stats(), (50, 10, 18.0, 1.2));
    assert_eq!(EnemyKind::Boss.stats(), (100, 50, 12.0, 1.8));
}

#[test]
fn tower_kind_costs_are_expected() {
    assert_eq!(TowerKind::Bow.cost(), (3, 1));
    assert_eq!(TowerKind::Crossbow.cost(), (10, 3));
}
