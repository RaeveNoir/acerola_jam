#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables))]
use bevy::prelude::*;

pub struct ParticlePlugin;
impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin)
    }
}
