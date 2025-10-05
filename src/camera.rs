use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
};

use crate::character::{LocalPlayer, Player};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                camera_center_player,
                pan_orbit_camera.run_if(any_with_component::<PanOrbitState>),
            )
                .chain(),
        );
    }
}

// Bundle to spawn our custom camera easily
#[derive(Bundle, Default)]
pub struct PanOrbitCameraBundle {
    pub camera: Camera3d,
    pub state: PanOrbitState,
    pub settings: PanOrbitSettings,
}

// The internal state of the pan-orbit controller
#[derive(Component)]
pub struct PanOrbitState {
    pub center: Vec3,
    pub radius: f32,
    pub upside_down: bool,
    pub pitch: f32,
    pub yaw: f32,
}

/// The configuration of the pan-orbit controller
#[derive(Component)]
pub struct PanOrbitSettings {
    /// World units per pixel of mouse motion
    pub pan_sensitivity: f32,
    /// Radians per pixel of mouse motion
    pub orbit_sensitivity: f32,
    /// Exponent per pixel of mouse motion
    pub zoom_sensitivity: f32,
    /// Key to hold for panning
    pub pan_key: Option<KeyCode>,
    /// Key to hold for orbiting
    pub orbit_key: Option<MouseButton>,
    /// Key to hold for zooming
    pub zoom_key: Option<KeyCode>,
    /// What action is bound to the scroll wheel?
    pub scroll_action: Option<PanOrbitAction>,
    /// For devices with a notched scroll wheel, like desktop mice
    pub scroll_line_sensitivity: f32,
    /// For devices with smooth scrolling, like touchpads
    pub scroll_pixel_sensitivity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanOrbitAction {
    Pan,
    Orbit,
    Zoom,
}

impl Default for PanOrbitState {
    fn default() -> Self {
        PanOrbitState {
            center: Vec3::ZERO,
            radius: 1.0,
            upside_down: false,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

impl Default for PanOrbitSettings {
    fn default() -> Self {
        PanOrbitSettings {
            pan_sensitivity: 0.001,                 // 1000 pixels per world unit
            orbit_sensitivity: 0.1f32.to_radians(), // 0.1 degree per pixel
            zoom_sensitivity: 0.01,
            pan_key: Some(KeyCode::ControlLeft),
            orbit_key: Some(MouseButton::Right),
            zoom_key: Some(KeyCode::ShiftLeft),
            scroll_action: Some(PanOrbitAction::Zoom),
            scroll_line_sensitivity: 16.0, // 1 "line" == 16 "pixels of motion"
            scroll_pixel_sensitivity: 1.0,
        }
    }
}

pub fn camera_center_player(
    player_query: Single<&Transform, With<LocalPlayer>>,
    camera: Single<&mut PanOrbitState>,
) {
    let transform = player_query.into_inner();
    let mut camera = camera.into_inner();
    camera.center = transform.translation;
}

pub fn spawn_camera(mut commands: Commands) {
    let mut camera = PanOrbitCameraBundle::default();
    camera.state.center = Vec3::new(1.0, 2.0, 3.0);
    camera.state.radius = 50.0;
    camera.state.pitch = 15.0f32.to_radians();
    camera.state.yaw = 30.0f32.to_radians();
    commands.spawn(camera);
}

pub fn pan_orbit_camera(
    kbd: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut evr_motion: MessageReader<MouseMotion>,
    mut evr_scroll: MessageReader<MouseWheel>,
    mut q_camera: Query<(&PanOrbitSettings, &mut PanOrbitState, &mut Transform)>,
) {
    // First, accumulate the total amount of
    // mouse motion and scroll, from all pending events:
    let total_motion: Vec2 = evr_motion.read().map(|ev| ev.delta).sum();

    // Reverse Y (Bevy's Worldspace coordinate system is Y-Up,
    // but events are in window/ui coordinates, which are Y-Down)
    // total_motion.y = -total_motion.y;

    let mut total_scroll_lines = Vec2::ZERO;
    let mut total_scroll_pixels = Vec2::ZERO;
    for ev in evr_scroll.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                total_scroll_lines.x += ev.x;
                total_scroll_lines.y -= ev.y;
            }
            MouseScrollUnit::Pixel => {
                total_scroll_pixels.x += ev.x;
                total_scroll_pixels.y -= ev.y;
            }
        }
    }

    for (settings, mut state, mut transform) in &mut q_camera {
        // Check how much of each thing we need to apply.
        // Accumulate values from motion and scroll,
        // based on our configuration settings.

        let mut total_pan = Vec2::ZERO;
        if settings
            .pan_key
            .map(|key| kbd.pressed(key))
            .unwrap_or(false)
        {
            total_pan -= total_motion * settings.pan_sensitivity;
        }
        if settings.scroll_action == Some(PanOrbitAction::Pan) {
            total_pan -=
                total_scroll_lines * settings.scroll_line_sensitivity * settings.pan_sensitivity;
            total_pan -=
                total_scroll_pixels * settings.scroll_pixel_sensitivity * settings.pan_sensitivity;
        }

        let mut total_orbit = Vec2::ZERO;
        if settings
            .orbit_key
            .map(|key| mouse.pressed(key))
            .unwrap_or(false)
        {
            total_orbit -= total_motion * settings.orbit_sensitivity;
        }
        if settings.scroll_action == Some(PanOrbitAction::Orbit) {
            total_orbit -=
                total_scroll_lines * settings.scroll_line_sensitivity * settings.orbit_sensitivity;
            total_orbit -= total_scroll_pixels
                * settings.scroll_pixel_sensitivity
                * settings.orbit_sensitivity;
        }

        let mut total_zoom = Vec2::ZERO;
        if settings
            .zoom_key
            .map(|key| kbd.pressed(key))
            .unwrap_or(false)
        {
            total_zoom -= total_motion * settings.zoom_sensitivity;
        }
        if settings.scroll_action == Some(PanOrbitAction::Zoom) {
            total_zoom -=
                total_scroll_lines * settings.scroll_line_sensitivity * settings.zoom_sensitivity;
            total_zoom -=
                total_scroll_pixels * settings.scroll_pixel_sensitivity * settings.zoom_sensitivity;
        }

        // Upon starting a new orbit maneuver (key is just pressed),
        // check if we are starting it upside-down
        if settings
            .orbit_key
            .map(|key| mouse.just_pressed(key))
            .unwrap_or(false)
        {
            state.upside_down = state.pitch < -FRAC_PI_2 || state.pitch > FRAC_PI_2;
        }

        // If we are upside down, reverse the X orbiting
        if state.upside_down {
            total_orbit.x = -total_orbit.x;
        }

        // Now we can actually do the things!

        let mut any = false;

        // To ZOOM, we need to multiply our radius.
        if total_zoom != Vec2::ZERO {
            any = true;
            // in order for zoom to feel intuitive,
            // everything needs to be exponential
            // (done via multiplication)
            // not linear
            // (done via addition)

            // so we compute the exponential of our
            // accumulated value and multiply by that
            state.radius *= (-total_zoom.y).exp();
        }

        // To ORBIT, we change our pitch and yaw values
        if total_orbit != Vec2::ZERO {
            any = true;
            state.yaw += total_orbit.x;
            state.pitch += total_orbit.y;
            // wrap around, to stay between +- 180 degrees
            if state.yaw > PI {
                state.yaw -= TAU; // 2 * PI
            }
            if state.yaw < -PI {
                state.yaw += TAU; // 2 * PI
            }
            if state.pitch > PI {
                state.pitch -= TAU; // 2 * PI
            }
            if state.pitch < -PI {
                state.pitch += TAU; // 2 * PI
            }
        }

        // To PAN, we can get the UP and RIGHT direction
        // vectors from the camera's transform, and use
        // them to move the center point. Multiply by the
        // radius to make the pan adapt to the current zoom.
        if total_pan != Vec2::ZERO {
            any = true;
            let radius = state.radius;
            state.center += transform.right() * total_pan.x * radius;
            state.center += transform.up() * total_pan.y * radius;
        }

        // Finally, compute the new camera transform.
        // (if we changed anything, or if the pan-orbit
        // controller was just added and thus we are running
        // for the first time and need to initialize)
        // if any || state.is_added() {
        // YXZ Euler Rotation performs yaw/pitch/roll.
        transform.rotation = Quat::from_euler(EulerRot::YXZ, state.yaw, state.pitch, 0.0);
        // To position the camera, get the backward direction vector
        // and place the camera at the desired radius from the center.
        transform.translation = state.center + transform.back() * state.radius;
        // }
    }
}

// use std::{f32::consts::FRAC_PI_2, ops::Range}
//
// #[derive(Resource)]
// pub enum CameraMode {
//     Track,
//     Chase,
// }
//
// #[derive(Component)]
// pub struct CameraTracked;
//
// #[derive(Resource)]
// pub struct CameraSettings {
//     pub orbit_distance: f32,
//     pub pitch_speed: f32,
//     pub pitch_range: Range<f32>,
//     pub roll_speed: f32,
//     pub yaw_speed: f32,
// }
//
// impl Default for CameraSettings {
//     fn default() -> Self {
//         let pitch_limit = FRAC_PI_2 - 0.01;
//         Self {
//             orbit_distance: 20.0,
//             pitch_speed: 0.003,
//             pitch_range: -pitch_limit..pitch_limit,
//             roll_speed: 1.0,
//             yaw_speed: 0.004,
//         }
//     }
// }
//
// pub fn move_camera(
//     camera: Single<(&mut Transform, &mut Projection), Without<CameraTracked>>,
//     tracked: Single<&Transform, With<CameraTracked>>,
//     mode: Res<CameraMode>,
// ) {
//     let (mut transform, mut projection) = camera.into_inner();
//     match *mode {
//         CameraMode::Track => {
//             transform.look_at(tracked.translation, Vec3::Y);
//             transform.translation = Vec3::new(15.0, -0.5, 0.0);
//             if let Projection::Perspective(perspective) = &mut *projection {
//                 perspective.fov = 0.05;
//             }
//         }
//         CameraMode::Chase => {
//             transform.translation =
//                 tracked.translation + Vec3::new(0.0, 0.15, 0.0) + tracked.back() * 0.6;
//             transform.look_to(tracked.forward(), Vec3::Y);
//             if let Projection::Perspective(perspective) = &mut *projection {
//                 perspective.fov = 1.0;
//             }
//         }
//     }
// }
//
// pub fn setup_camera(mut commands: Commands) {
//     commands.spawn(Camera3d::default());
// }

// #[derive(Component)]
// pub struct CameraTarget;
//
// #[derive(Component)]
// pub struct CameraController {
//     pub zoom_speed: f32,
//     pub pan_sensitivity: f32,
//     pub min_distance: f32,
//     pub max_distance: f32,
//     pub pan_offset: Vec3,
//     pub pitch: f32,
//     pub yaw: f32,
//     pub distance: f32,
// }
//
// #[derive(Component)]
// pub struct Camera;
//
// pub fn camera_follow(
//     mut camera_query: Query<&mut Transform, With<CameraTarget>>,
//     target_query: Query<&Transform, (With<CameraTarget>, Without<Camera>)>,
//     controller: Query<&CameraController>,
// ) {
//     if let (Ok(mut camera_transform), Ok(target_transform), Ok(controller)) = (
//         camera_query.single_mut(),
//         target_query.single(),
//         controller.single(),
//     ) {
//         let offset = Quat::from_euler(EulerRot::YXZ, controller.yaw, controller.pitch, 0.)
//             * Vec3::new(0., 0., controller.distance);
//         let target_pos = target_transform.translation + controller.pan_offset + offset;
//         camera_transform.translation = target_pos;
//         camera_transform.look_at(
//             target_transform.translation + controller.pan_offset,
//             Vec3::Y,
//         );
//     }
// }
//
// pub fn setup_camera(mut commands: Commands) {
//     commands.spawn((
//         Camera3d::default(),
//         CameraController {
//             zoom_speed: 1.0,
//             pan_sensitivity: 1.0,
//             min_distance: 2.0,
//             max_distance: 50.0,
//             distance: 10.0,
//             pan_offset: Vec3::ZERO,
//             pitch: 0.0,
//             yaw: 0.0,
//         },
//         Transform::from_xyz(0., 5., 10.),
//     ));
// }
