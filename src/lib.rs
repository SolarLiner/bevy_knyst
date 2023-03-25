use bevy::asset::HandleId;
use std::fmt::Formatter;
use std::{fmt, ops};

use bevy::prelude::*;
use bevy::utils::HashMap;
use knyst::audio_backend::CpalBackendOptions;
use knyst::controller::KnystCommands;
use knyst::{audio_backend::CpalBackend, graph::NodeAddress, prelude::*, BufferId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub struct AudioGraphProcessing;

#[derive(Debug, Clone, Component)]
pub struct NodeRef(pub NodeAddress);

impl ops::Deref for NodeRef {
    type Target = NodeAddress;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct KnystBackend(pub CpalBackend);

impl ops::Deref for KnystBackend {
    type Target = CpalBackend;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for KnystBackend {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromWorld for KnystBackend {
    fn from_world(world: &mut World) -> Self {
        let settings = world
            .remove_non_send_resource::<CpalBackendOptions>()
            .unwrap_or_default();
        Self(CpalBackend::new(settings).unwrap())
    }
}

#[derive(Clone, Resource)]
pub struct AudioGraphCommands(KnystCommands);

impl ops::Deref for AudioGraphCommands {
    type Target = KnystCommands;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for AudioGraphCommands {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Debug for AudioGraphCommands {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AudioGraphCommands").field(&"..").finish()
    }
}

pub struct AudioHandleRef {
    buffer: BufferId,
    handle: Handle<AudioSource>,
}

#[derive(Default)]
pub struct AudioGraphPlugin;

impl Plugin for AudioGraphPlugin {
    fn build(&self, app: &mut App) {
        app.world.init_non_send_resource::<KnystBackend>();
        let resources = Resources::new(app.world.remove_non_send_resource().unwrap_or_default());
        let run_graph_settings = app.world.remove_non_send_resource().unwrap_or_default();
        let graph_settings = app.world.remove_non_send_resource().unwrap_or_default();
        let mut graph_backend = app
            .world
            .get_non_send_resource_mut::<KnystBackend>()
            .unwrap();
        let graph = Graph::new(graph_settings);
        let commands = graph_backend
            .start_processing(graph, resources, run_graph_settings, |error| {
                error!("Knyst error: {}", error);
            })
            .unwrap();
        app.insert_resource(AudioGraphCommands(commands));
    }
}
