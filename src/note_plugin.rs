use bevy::{
    input::keyboard::KeyboardInput,
    math::{bool, sampling::shape_sampling},
    prelude::*,
    render::view::visibility,
    state::commands,
};

const HIT_MARGIN: f32 = 0.15;
const NOTE_SPEED: f32 = 5.0;

pub struct NotePlugin;

impl Plugin for NotePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup, load_song));
        app.add_systems(Update, move_notes_down);
        app.add_systems(Update, spawn_notes_from_song);
        app.add_systems(
            Update,
            (evaluate_notes, hit_note, handle_missed_notes).chain(),
        );
        app.add_systems(Update, illuminate_lane);
    }
}

#[derive(Component)]
struct Lane(u8);

#[derive(Component)]
struct Song {
    pub bpm: f32,
}

#[derive(Component)]
struct NoteData {
    lane: u8,
    time: f32,
}

#[derive(Component)]
struct Note {
    lane: u8,
    time: f32,
}

#[derive(Component)]
struct Spawned;

fn load_song(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for i in 0..=23 {
        commands.spawn(NoteData {
            lane: i % 4,
            time: 4. + i as f32,
        });
    }

    let lane_width = 50.;
    let lane_spacing = 50.;

    for i in 0..=3 {
        let shape = meshes.add(Circle::new(lane_width));
        let color = materials.add(Color::hsl(0., 0., 0.3));
        commands.spawn((
            Lane(i),
            Transform::from_xyz(i as f32 * (2. * lane_width + lane_spacing), -500., 0.),
            Mesh2d(shape),
            MeshMaterial2d(color),
        ));
    }

    commands.spawn(Song { bpm: 4.0 });
}

fn spawn_notes_from_song(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut query: Query<(Entity, &NoteData), Without<Spawned>>,
) {
    let current_time = time.elapsed_secs();

    for (entity, note_data) in &mut query {
        if note_data.time - 5. <= current_time {
            println!("Spawning note for Entity: {:?}", entity);
            let shape = meshes.add(Circle::new(50.0));
            let color = materials.add(Color::hsl(250., 0.95, 0.7));

            commands.spawn(NoteBundle {
                mesh_2d: Mesh2d(shape),
                mesh_material_2d: MeshMaterial2d(color),
                transform: Transform::from_xyz(note_data.lane as f32 * 100., 1000., 1.),
                note: Note {
                    lane: note_data.lane,
                    time: note_data.time,
                },
                visibility: Visibility::Visible,
            });
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Bundle)]
struct NoteBundle {
    mesh_2d: Mesh2d,
    mesh_material_2d: MeshMaterial2d<ColorMaterial>,
    transform: Transform,
    note: Note,
    visibility: Visibility,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn move_notes_down(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Note), Without<Lane>>,
    query_lanes: Query<(&Transform, &Lane), Without<Note>>,
) {
    let current_time = time.elapsed_secs();

    for (mut transform, note) in &mut query {
        if let Some((lane_transform, _lane)) = query_lanes
            .iter()
            .find(|(_lane_transform, lane)| lane.0 == note.lane)
        {
            let hit_position = (lane_transform.translation.x, lane_transform.translation.y);
            transform.translation.x = hit_position.0;
            transform.translation.y = hit_position.1 - (current_time - note.time) * 200.;
        } else {
            panic!("Note is in unexpected lane {}", note.lane);
        }
    }
}

#[derive(Component)]
struct IsEvaluable;

#[derive(Component)]
struct HitResult {
    is_hit: bool,
    hit_time: f32,
    offset: f32,
}

fn evaluate_notes(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &Note), (Without<IsEvaluable>, Without<HitResult>)>,
) {
    let current_time = time.elapsed_secs();

    for (entity, note) in &mut query {
        if (note.time - current_time).abs() < HIT_MARGIN {
            println!("Marked note as evaluable");
            commands.entity(entity).insert(IsEvaluable);
        }
    }
}

fn hit_note(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(Entity, &Note, &mut Visibility), With<IsEvaluable>>,
) {
    let current_time = time.elapsed_secs();

    for (entity, note, mut visibility) in &mut query {
        println!("HIT NOTE INSIDE FUNCTION");

        let key_code = match note.lane {
            0 => KeyCode::KeyD,
            1 => KeyCode::KeyF,
            2 => KeyCode::KeyJ,
            3 => KeyCode::KeyK,
            _ => panic!("Unexpected lane: {}", note.lane),
        };

        if input.just_pressed(key_code) {
            commands.entity(entity).remove::<IsEvaluable>();
            commands.entity(entity).insert(HitResult {
                is_hit: true,
                hit_time: current_time,
                offset: current_time - note.time,
            });
            *visibility = Visibility::Hidden;
            println!("TOGGLE Visibility lane {}", note.lane);
        }
    }
}

fn handle_missed_notes(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &Note), With<IsEvaluable>>,
) {
    let current_time = time.elapsed_secs();

    for (entity, note) in &mut query {
        if current_time > note.time + HIT_MARGIN {
            println!("Marked as failed note {}", note.time);
            commands.entity(entity).remove::<IsEvaluable>();
            commands.entity(entity).insert(HitResult {
                is_hit: false,
                hit_time: current_time,
                offset: current_time - note.time,
            });
        }
    }
}

fn illuminate_lane(
    mut query: Query<(&mut Lane, &mut MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (lane, mesh) in &mut query {
        let material = materials.get_mut(&mesh.0).unwrap();
        match lane.0 {
            0 => {
                if input.just_pressed(KeyCode::KeyD) {
                    material.color = Color::hsl(0., 0., 1.); // Cambia a rojo
                } else if input.just_released(KeyCode::KeyD) {
                    material.color = Color::hsl(0., 0., 0.3); // Cambia a color original
                }
            }
            1 => {
                if input.just_pressed(KeyCode::KeyF) {
                    material.color = Color::hsl(0., 0., 1.); // Cambia a rojo
                } else if input.just_released(KeyCode::KeyF) {
                    material.color = Color::hsl(0., 0., 0.3); // Cambia a color original
                }
            }
            2 => {
                if input.just_pressed(KeyCode::KeyJ) {
                    material.color = Color::hsl(0., 0., 1.); // Cambia a rojo
                } else if input.just_released(KeyCode::KeyJ) {
                    material.color = Color::hsl(0., 0., 0.3); // Cambia a color original
                }
            }
            3 => {
                if input.just_pressed(KeyCode::KeyK) {
                    material.color = Color::hsl(0., 0., 1.); // Cambia a rojo
                } else if input.just_released(KeyCode::KeyK) {
                    material.color = Color::hsl(0., 0., 0.3); // Cambia a color original
                }
            }
            _ => panic!("Unexpected lane: {}", lane.0),
        }
    }
}
