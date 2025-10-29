use td::random_policy::RandomizationPolicy;

#[test]
fn randomization_policy_defaults_are_seeded() {
    let p = RandomizationPolicy::default();
    assert!(p.wave_composition_seeded);
    assert!(p.enemy_spawn_selection_seeded);
    assert!(p.town_layout_seeded);
    assert!(p.road_generation_seeded);
    assert!(p.chunk_content_seeded);
    assert!(p.resource_rules_seeded);
}
