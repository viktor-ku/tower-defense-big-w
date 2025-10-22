use crate::components::*;
use crate::events::*;
use bevy::prelude::*;

pub fn tree_collection(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&Transform, &mut Player)>,
    mut tree_query: Query<(&Transform, &mut Tree)>,
    mut wood_events: MessageWriter<WoodCollected>,
) {
    if !keyboard_input.pressed(KeyCode::KeyE) {
        return;
    }

    if let Ok((player_transform, mut player)) = player_query.single_mut() {
        for (tree_transform, mut tree) in tree_query.iter_mut() {
            // Check if player is near tree and tree has wood
            let distance = player_transform
                .translation
                .distance(tree_transform.translation);

            if distance < 25.0 && tree.wood_amount > 0 {
                // Collect wood from tree
                let collected = (tree.wood_amount).min(5); // Collect up to 5 wood per press
                tree.wood_amount -= collected;
                player.wood += collected;

                // Emit event
                wood_events.write(WoodCollected {
                    amount: collected,
                    tree_position: tree_transform.translation,
                });

                // If tree is empty, mark as chopped
                if tree.wood_amount == 0 {
                    tree.is_chopped = true;
                    info!("Tree chopped down!");
                }

                info!("Collected {} wood! Total wood: {}", collected, player.wood);
                break; // Only collect from one tree at a time
            }
        }
    }
}

pub fn handle_wood_collected_events(mut wood_events: MessageReader<WoodCollected>) {
    for event in wood_events.read() {
        info!(
            "Wood collected: {} at position {:?}",
            event.amount, event.tree_position
        );
    }
}
