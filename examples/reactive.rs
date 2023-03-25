use bevy::prelude::*;
use knyst::{
    graph::{ConnectionError, NodeAddress},
    prelude::*,
    wavetable::WavetableOscillatorOwned,
};

use bevy_knyst::{AudioGraphPlugin, NodeRef};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
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

fn setup(mut commands: Commands, mut graph: NonSendMut<Graph>, asset_server: Res<AssetServer>) {
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
            let mut runner = || -> Result<NodeAddress, ConnectionError> {
                let carrier = graph.push_gen(WavetableOscillatorOwned::new(Wavetable::sine()));
                let amp = graph.push_gen(Mult);
                let amp_amt = graph.push(constant(0.));
                graph.connect(carrier.to(amp).to_index(0))?;
                graph.connect(constant(0.).to(amp).to_index(1))?;
                graph.connect(constant(440.).to(carrier).to_label("freq"))?;
                graph.connect(amp.to_graph_out())?;
                graph.connect(amp.to_graph_out().to_index(1))?;
                Ok(amp)
            };
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
                            ..default()
                        },
                    ));
                })
                .insert((NodeRef(runner().unwrap()), OnOffSine::default()));
        });
}

fn toggle_audio_button(
    mut interaction_query: Query<
        (&Interaction, &Children, &NodeRef, &mut OnOffSine),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut Text>,
    mut graph: NonSendMut<Graph>,
) {
    for (interaction, children, NodeRef(node), mut onoff) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                let mut text = text_query.get_mut(children[0]).unwrap();
                onoff.is_on = !onoff.is_on;
                graph
                    .schedule_change(ParameterChange::now(
                         node,
                        if onoff.is_on { 1. } else { 0. },
                    ))
                    .unwrap();
                info!("Audio is now {:?}", onoff);
                text.sections[0].value =
                    if onoff.is_on { "Turn Off" } else { "Turn On" }.to_string();
            }
            _ => {}
        }
    }
}
