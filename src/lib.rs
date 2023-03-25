
use std::ops;


use bevy::prelude::*;
use knyst::{
    audio_backend::{CpalBackend},
    graph::NodeAddress,
    prelude::*,
};

#[derive(Debug, StageLabel)]
pub enum AudioStage {
    PreGraphProcessing,
    AudioGraphProcessing,
    PostGraphProcessing,
}

#[derive(Debug, Copy, Clone, Component)]
pub struct NodeRef(pub NodeAddress);

impl ops::Deref for NodeRef {
    type Target = NodeAddress;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct AudioGraphBackend {
    pub backend: CpalBackend,
}

impl FromWorld for AudioGraphBackend {
    fn from_world(world: &mut World) -> Self {
        let settings = world
            .get_non_send_resource_mut()
            .map(|mut v| std::mem::take(&mut *v))
            .unwrap_or_default();
        Self {
            backend: CpalBackend::new(settings).unwrap(),
        }
    }
}

#[derive(Default)]
pub struct AudioGraphPlugin;

impl Plugin for AudioGraphPlugin {
    fn build(&self, app: &mut App) {
        app.world.init_non_send_resource::<AudioGraphBackend>();
        let graph_settings = app
            .world
            .get_non_send_resource_mut::<GraphSettings>()
            .map(|mut v| std::mem::take(&mut *v))
            .unwrap_or_default();
        let graph = Graph::new(graph_settings);
        app.insert_non_send_resource(graph)
            .add_startup_system(start_audio_backend)
            .add_stage(AudioStage::AudioGraphProcessing, SystemStage::parallel())
            .add_stage_before(
                AudioStage::AudioGraphProcessing,
                AudioStage::PreGraphProcessing,
                SystemStage::parallel(),
            )
            .add_stage_after(
                AudioStage::AudioGraphProcessing,
                AudioStage::PostGraphProcessing,
                SystemStage::parallel(),
            )
            .add_system_to_stage(AudioStage::AudioGraphProcessing, commit_graph_changes);
    }

    fn is_unique(&self) -> bool {
        true
    }
}

fn start_audio_backend(mut backend: NonSendMut<AudioGraphBackend>, mut graph: NonSendMut<Graph>) {
    backend
        .backend
        .start_processing(&mut *graph, Resources::new(default()))
        .expect("Could not start audio graph");
}

fn commit_graph_changes(mut graph: NonSendMut<Graph>) {
    if graph.is_changed() {
        debug!("Commiting changes to audio graph");
        graph.commit_changes();
        graph.update();
    }
}
