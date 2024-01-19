use bevy::{prelude::*, input::mouse::MouseMotion, window::{PrimaryWindow, CursorGrabMode}};

#[derive(Component)]
pub struct CameraMarker;

#[derive(Bundle)]
pub struct Camera {
    marker: CameraMarker,
    cam: Camera3dBundle,
    config: CameraConfig
}
impl Camera {
    pub fn spawn(parent: &mut ChildBuilder) {
        parent.spawn(Self {
            marker: CameraMarker,
            cam: Camera3dBundle {
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..Default::default()
            },
            config: CameraConfig::default()
        });
    }
}

#[derive(Component)]
pub struct CameraConfig {
    sensi_x: f32,
    sensi_y: f32
}
impl Default for CameraConfig {
    fn default() -> Self {
        CameraConfig { sensi_x: 0.01, sensi_y: 0.01 }
    }
}

pub fn rotate_camera(
    mut motion_evr: EventReader<MouseMotion>,
    mut cam: Query<(&mut Transform, &CameraConfig), With<CameraMarker>>
) {
    let (mut cam_pos, config) = cam.single_mut();
    for ev in motion_evr.read() {
        let mov = ev.delta;
        cam_pos.rotate_y(-mov.x * config.sensi_x);
        cam_pos.rotate_local_x(-mov.y * config.sensi_y);
    }
}

pub fn cursor_grab(
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut primary_window = q_windows.single_mut();

    // for a game that doesn't use the cursor (like a shooter):
    // use `Locked` mode to keep the cursor in one place
    primary_window.cursor.grab_mode = CursorGrabMode::Locked;

    // also hide the cursor
    primary_window.cursor.visible = false;
}