use std::time::Duration;

use bevy::{
    app::{RunMode, ScheduleRunnerSettings},
    prelude::*
};
use knyst::{prelude::*, wavetable::WavetableOscillatorOwned};

use bevy_knyst::{AudioGraphCommands, AudioGraphPlugin, AudioGraphProcessing, NodeRef};

#[derive(Component)]
struct MySine;

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings {
            run_mode: RunMode::Loop {
                wait: Some(Duration::from_micros(16_667)),
            },
        })
        .add_plugins(MinimalPlugins)
        .add_plugin(AudioGraphPlugin)
        .add_startup_system(add_sine)
        .add_system(modulate_sine.before(AudioGraphProcessing))
        .run();
}

fn add_sine(mut commands: Commands, mut graph: ResMut<AudioGraphCommands>) {
    let carrier = graph.push(
        WavetableOscillatorOwned::new(Wavetable::sine()),
        inputs![("freq" : 440.)],
    );
    graph.connect(carrier.to_graph_out());
    graph.connect(carrier.to_graph_out().to_index(1));
    commands.spawn((NodeRef(carrier), MySine));
}

fn modulate_sine(time: Res<Time>, mut graph: ResMut<AudioGraphCommands>, q: Query<&NodeRef, With<MySine>>) {
    let freq = time.elapsed_seconds().sin() * 100. + 440.;
    for node in &q {
        graph
            .schedule_change(ParameterChange::now(node.0.clone(), freq));
    }
}
