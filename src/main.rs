use bevy::prelude::*;
use note_plugin::NotePlugin;

mod note_plugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NotePlugin)
        .run();
}
