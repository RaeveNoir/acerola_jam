#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables))]
use bevy::prelude::*;
use bevy_hanabi::prelude::*;

pub struct ParticlePlugin;
impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin)
            .add_systems(Startup, particle_setup);
    }
}

fn particle_setup(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    let writer = ExprWriter::new();
    let accelmod = writer.lit(Vec3::new(0.0, -0.45, 0.0)).expr();
    let height = writer.lit(100.0).expr();
    let base_radius = writer.lit(0.0).expr();
    let top_radius = writer.lit(0.0).expr();
    let effect = effects.add(
        EffectAsset::new(
            vec![32768],
            Spawner::once(32.0.into(), true),
            writer.finish(),
        )
        // name: "Petals".into(),
        .init(SetPositionCone3dModifier {
            height,
            base_radius,
            top_radius,
            dimension: bevy_hanabi::ShapeDimension::Volume,
        })
        .update(AccelModifier::new(accelmod)),
    );

    commands
        .spawn(ParticleEffectBundle::new(effect))
        .insert(Name::new("effect"));
}
