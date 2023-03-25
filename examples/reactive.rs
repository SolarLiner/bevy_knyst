use bevy::prelude::*;
use knyst::{prelude::*, wavetable::WavetableOscillatorOwned};
use std::time::Duration;

use bevy_knyst::{AudioGraphCommands, AudioGraphPlugin, NodeRef};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_non_send_resource(RunGraphSettings {
            scheduling_latency: Duration::from_millis(100),
        })
        .add_plugin(AudioGraphPlugin)
        .add_startup_system(setup)
        .add_system(toggle_audio_button)
        .run();
}

#[derive(Component, Debug)]
struct OnOffSine {
    is_on: bool,
}

impl Default for OnOffSine {
    fn default() -> Self {
        Self { is_on: true }
    }
}

fn setup(
    mut commands: Commands,
    mut graph: ResMut<AudioGraphCommands>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default());

    // UI
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            let carrier = graph.push(
                WavetableOscillatorOwned::new(Wavetable::sine()),
                inputs![("freq": 440.)],
            );
            let amp = graph.push(Mult, inputs![(0 ; carrier.out(0)), (1: 0.)]);

            graph.connect(amp.to_graph_out());
            graph.connect(amp.to_graph_out().to_index(1));
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                        // center button
                        margin: UiRect::all(Val::Auto),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::WHITE),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Turn Off",
                        TextStyle {
                            font_size: 40.,
                            font: asset_server.load("fonts/BebasNeue-Regular.ttf"),
                            color: Color::BLACK,
                        },
                    ));
                })
                .insert((NodeRef(amp), OnOffSine::default()));
        });
}

fn toggle_audio_button(
    mut interaction_query: Query<
        (&Interaction, &Children, &NodeRef, &mut OnOffSine),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut Text>,
    mut graph: ResMut<AudioGraphCommands>,
) {
    for (interaction, children, NodeRef(node), mut onoff) in &mut interaction_query {
        if *interaction == Interaction::Clicked {
            let mut text = text_query.get_mut(children[0]).unwrap();
            onoff.is_on = !onoff.is_on;
            graph.schedule_change(ParameterChange::now(
                node.clone(),
                if onoff.is_on { 1. } else { 0. },
            ));
            info!("Audio is now {:?}", onoff);
            text.sections[0].value =
                if onoff.is_on { "Turn Off" } else { "Turn On" }.to_string();
        }
    }
}
