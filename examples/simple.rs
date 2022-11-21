use std::time::Duration;
use bevy::{
    prelude::*,
    time::TimePlugin
};
use bevy::app::{RunMode, ScheduleRunnerSettings};
use knyst::{
    audio_backend::CpalBackend,
    graph::Gen,
    prelude::*,
    wavetable::WavetableOscillatorOwned
};
use knyst::graph::{ConnectionError, NodeAddress};

use bevy_audio_graph::{AudioGraphPlugin, AudioStage, NodeRef};

#[derive(Component)]
struct MySine;

fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings {run_mode: RunMode::Loop {wait: Some(Duration::from_micros(16_667))}})
        .add_plugins(MinimalPlugins)
        .add_plugin(AudioGraphPlugin)
        .add_startup_system(add_sine)
        .add_system_to_stage(AudioStage::PreGraphProcessing, modulate_sine)
        .run();
}

fn add_sine(mut commands: Commands, mut graph: NonSendMut<Graph>) {
    let mut runner = || -> Result<NodeAddress, ConnectionError> {
        let sine_node = graph.push_gen(WavetableOscillatorOwned::new(Wavetable::sine()));
        graph.connect(constant(440.).to(sine_node).to_label("freq"))?;
        let modulator = graph.push_gen(WavetableOscillatorOwned::new(Wavetable::sine()));
        graph.connect(constant(5.).to(modulator).to_label("freq"))?;
        let mod_amp = graph.push_gen(Mult);
        graph.connect(modulator.to(mod_amp))?;
        graph.connect(constant(0.25).to(mod_amp).to_index(1))?;
        let amp = graph.push_gen(Mult);
        graph.connect(sine_node.to(amp))?;
        graph.connect(constant(0.5).to(amp).to_index(1))?;
        graph.connect(mod_amp.to(amp).to_index(1))?;
        graph.connect(amp.to_graph_out())?;
        graph.connect(amp.to_graph_out().to_index(1))?;
        Ok(sine_node)
    };
    commands.spawn((NodeRef(runner().unwrap()), MySine));
}

fn modulate_sine(time: Res<Time>, mut graph: NonSendMut<Graph>, q: Query<&NodeRef, With<MySine>>) {
    let amp = time.elapsed_seconds().sin() * 0.5 + 0.5;
    for node in &q {
        graph
            .schedule_change(ParameterChange::now(**node, amp))
            .unwrap();
    }
}
