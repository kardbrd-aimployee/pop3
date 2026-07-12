use winit::keyboard::KeyCode;

use crate::engine::objects::ObjectHandle;
use crate::render::hud::HudTab;

/// Input boundary — every action the game engine can process,
/// regardless of source (keyboard, mouse, script, network, test harness).
#[derive(Debug, Clone)]
pub enum AppCommand {
    // Camera / view
    RotateCamera {
        delta_z: i16,
    },
    TiltCamera {
        delta_x: i16,
    },
    /// Screen-relative pan: forward/right in screen space.
    /// Resolved to grid shifts using camera.angle_z in apply_command.
    PanScreen {
        forward: f32,
        right: f32,
    },
    /// Direct grid shift (HJKL-style, not screen-relative).
    PanTerrain {
        dx: i32,
        dy: i32,
    },
    ResetCamera,
    TopDownView,
    CenterOnShaman,
    SetZoom(f32),

    // Curvature
    ToggleCurvature,
    AdjustCurvature {
        factor: f32,
    },

    // Level navigation
    NextLevel,
    PrevLevel,

    // Shader / rendering toggles
    NextShader,
    PrevShader,
    ToggleObjects,
    ToggleShadows,
    ToggleMarkers,

    // Sunlight
    AdjustSunlight {
        dx: f32,
        dy: f32,
    },

    // Debug: sprite tuning
    AdjustSpriteOffset {
        delta: f32,
    },
    AdjustSpriteScale {
        delta: f32,
    },

    // Unit interaction (resolved game-level concepts, not raw screen coords)
    SelectUnit(ObjectHandle),
    SelectMultiple(Vec<ObjectHandle>),
    ClearSelection,
    OrderMove {
        x: f32,
        z: f32,
    },

    // Game state
    ToggleSimulation,
    IncreaseGameSpeed,
    DecreaseGameSpeed,

    // HUD
    SetHudTab(HudTab),
    ToggleHud,
    ToggleCompass,
    ToggleWalkability,

    // Building placement
    PlaceBuilding {
        building_type: u8,
        cell_x: i32,
        cell_y: i32,
        rotation: u8,
    },
    CancelPlacement,
    EnterBuildMode {
        building_type: u8,
    },
    // Lifecycle
    Quit,
}

/// Translate a winit KeyCode into a GameCommand.
/// Returns None for keys that have no game-command mapping.
pub fn translate_key(key: KeyCode) -> Option<AppCommand> {
    match key {
        // Orbit rotation
        KeyCode::KeyQ => Some(GameCommand::RotateCamera { delta_z: -5 }),
        KeyCode::KeyE => Some(GameCommand::RotateCamera { delta_z: 5 }),

        // Tilt
        KeyCode::ArrowUp => Some(GameCommand::TiltCamera { delta_x: 5 }),
        KeyCode::ArrowDown => Some(GameCommand::TiltCamera { delta_x: -5 }),

        // Screen-relative panning (WASD)
        KeyCode::KeyW => Some(GameCommand::PanScreen {
            forward: 1.0,
            right: 0.0,
        }),
        KeyCode::KeyS => Some(GameCommand::PanScreen {
            forward: -1.0,
            right: 0.0,
        }),
        KeyCode::KeyA => Some(GameCommand::PanScreen {
            forward: 0.0,
            right: -1.0,
        }),
        KeyCode::KeyD => Some(GameCommand::PanScreen {
            forward: 0.0,
            right: 1.0,
        }),

        // Direct grid panning (HJKL)
        KeyCode::KeyH => Some(GameCommand::PanTerrain { dx: 0, dy: -1 }),
        KeyCode::KeyL => Some(GameCommand::PanTerrain { dx: 0, dy: 1 }),
        KeyCode::KeyJ => Some(GameCommand::PanTerrain { dx: 1, dy: 0 }),
        KeyCode::KeyK => Some(GameCommand::PanTerrain { dx: -1, dy: 0 }),

        // Camera presets
        KeyCode::KeyR => Some(GameCommand::ResetCamera),
        KeyCode::KeyT => Some(GameCommand::TopDownView),
        KeyCode::Space => Some(GameCommand::CenterOnShaman),

        // Level navigation
        KeyCode::KeyB => Some(GameCommand::NextLevel),
        KeyCode::KeyV => Some(GameCommand::PrevLevel),

        // Shader cycling
        KeyCode::KeyN => Some(GameCommand::NextShader),
        KeyCode::KeyM => Some(GameCommand::PrevShader),

        // Toggles
        KeyCode::KeyC => Some(GameCommand::ToggleCurvature),
        KeyCode::KeyO => Some(GameCommand::ToggleObjects),
        KeyCode::KeyG => Some(GameCommand::ToggleShadows),
        KeyCode::KeyU => Some(GameCommand::ToggleMarkers),

        // Curvature adjustment
        KeyCode::BracketRight => Some(GameCommand::AdjustCurvature { factor: 1.2 }),
        KeyCode::BracketLeft => Some(GameCommand::AdjustCurvature { factor: 0.8 }),

        // Sunlight
        KeyCode::KeyY => Some(GameCommand::AdjustSunlight { dx: -1.0, dy: -1.0 }),

        // HUD
        KeyCode::F1 => Some(GameCommand::ToggleHud),
        KeyCode::F2 => Some(GameCommand::ToggleCompass),
        KeyCode::F8 => Some(GameCommand::ToggleWalkability),

        // Debug: sprite z-offset (F3 up / F4 down)
        KeyCode::F3 => Some(GameCommand::AdjustSpriteOffset { delta: 0.005 }),
        KeyCode::F4 => Some(GameCommand::AdjustSpriteOffset { delta: -0.005 }),
        // Debug: sprite scale (F6 bigger / F7 smaller)
        KeyCode::F6 => Some(GameCommand::AdjustSpriteScale { delta: 0.05 }),
        KeyCode::F7 => Some(GameCommand::AdjustSpriteScale { delta: -0.05 }),

        // Game simulation
        KeyCode::F5 => Some(GameCommand::ToggleSimulation),
        KeyCode::Equal => Some(GameCommand::IncreaseGameSpeed),
        KeyCode::Minus => Some(GameCommand::DecreaseGameSpeed),

        // Quit
        KeyCode::Escape => Some(GameCommand::Quit),

        _ => None,
    }
}

/// Compatibility alias while renderer call sites migrate to the explicit name.
pub type GameCommand = AppCommand;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_camera_rotation() {
        match translate_key(KeyCode::KeyQ) {
            Some(GameCommand::RotateCamera { delta_z: -5 }) => {}
            other => panic!("expected RotateCamera -5, got {:?}", other),
        }
        match translate_key(KeyCode::KeyE) {
            Some(GameCommand::RotateCamera { delta_z: 5 }) => {}
            other => panic!("expected RotateCamera 5, got {:?}", other),
        }
    }

    #[test]
    fn test_translate_tilt() {
        match translate_key(KeyCode::ArrowUp) {
            Some(GameCommand::TiltCamera { delta_x: 5 }) => {}
            other => panic!("expected TiltCamera 5, got {:?}", other),
        }
        match translate_key(KeyCode::ArrowDown) {
            Some(GameCommand::TiltCamera { delta_x: -5 }) => {}
            other => panic!("expected TiltCamera -5, got {:?}", other),
        }
    }

    #[test]
    fn test_translate_wasd_pan() {
        match translate_key(KeyCode::KeyW) {
            Some(GameCommand::PanScreen { forward, right }) => {
                assert_eq!(forward, 1.0);
                assert_eq!(right, 0.0);
            }
            other => panic!("expected PanScreen forward, got {:?}", other),
        }
        match translate_key(KeyCode::KeyA) {
            Some(GameCommand::PanScreen { forward, right }) => {
                assert_eq!(forward, 0.0);
                assert_eq!(right, -1.0);
            }
            other => panic!("expected PanScreen left, got {:?}", other),
        }
    }

    #[test]
    fn test_translate_hjkl_pan() {
        match translate_key(KeyCode::KeyH) {
            Some(GameCommand::PanTerrain { dx: 0, dy: -1 }) => {}
            other => panic!("expected PanTerrain dy=-1, got {:?}", other),
        }
        match translate_key(KeyCode::KeyL) {
            Some(GameCommand::PanTerrain { dx: 0, dy: 1 }) => {}
            other => panic!("expected PanTerrain dy=1, got {:?}", other),
        }
    }

    #[test]
    fn test_translate_level_nav() {
        assert!(matches!(
            translate_key(KeyCode::KeyB),
            Some(GameCommand::NextLevel)
        ));
        assert!(matches!(
            translate_key(KeyCode::KeyV),
            Some(GameCommand::PrevLevel)
        ));
    }

    #[test]
    fn test_translate_toggles() {
        assert!(matches!(
            translate_key(KeyCode::KeyC),
            Some(GameCommand::ToggleCurvature)
        ));
        assert!(matches!(
            translate_key(KeyCode::KeyO),
            Some(GameCommand::ToggleObjects)
        ));
        assert!(matches!(
            translate_key(KeyCode::KeyG),
            Some(GameCommand::ToggleShadows)
        ));
        assert!(matches!(
            translate_key(KeyCode::KeyU),
            Some(GameCommand::ToggleMarkers)
        ));
        assert!(matches!(
            translate_key(KeyCode::F5),
            Some(GameCommand::ToggleSimulation)
        ));
    }

    #[test]
    fn test_translate_curvature_adjust() {
        match translate_key(KeyCode::BracketRight) {
            Some(GameCommand::AdjustCurvature { factor }) => assert_eq!(factor, 1.2),
            other => panic!("expected AdjustCurvature 1.2, got {:?}", other),
        }
        match translate_key(KeyCode::BracketLeft) {
            Some(GameCommand::AdjustCurvature { factor }) => assert_eq!(factor, 0.8),
            other => panic!("expected AdjustCurvature 0.8, got {:?}", other),
        }
    }

    #[test]
    fn test_translate_presets() {
        assert!(matches!(
            translate_key(KeyCode::KeyR),
            Some(GameCommand::ResetCamera)
        ));
        assert!(matches!(
            translate_key(KeyCode::KeyT),
            Some(GameCommand::TopDownView)
        ));
        assert!(matches!(
            translate_key(KeyCode::Space),
            Some(GameCommand::CenterOnShaman)
        ));
    }

    #[test]
    fn test_translate_quit() {
        assert!(matches!(
            translate_key(KeyCode::Escape),
            Some(GameCommand::Quit)
        ));
    }

    #[test]
    fn test_translate_toggle_hud() {
        assert!(matches!(
            translate_key(KeyCode::F1),
            Some(GameCommand::ToggleHud)
        ));
    }

    #[test]
    fn test_translate_toggle_compass() {
        assert!(matches!(
            translate_key(KeyCode::F2),
            Some(GameCommand::ToggleCompass)
        ));
    }

    #[test]
    fn test_translate_toggle_walkability() {
        assert!(matches!(
            translate_key(KeyCode::F8),
            Some(GameCommand::ToggleWalkability)
        ));
    }

    #[test]
    fn test_translate_game_speed() {
        assert!(matches!(
            translate_key(KeyCode::Equal),
            Some(GameCommand::IncreaseGameSpeed)
        ));
        assert!(matches!(
            translate_key(KeyCode::Minus),
            Some(GameCommand::DecreaseGameSpeed)
        ));
    }

    #[test]
    fn test_translate_unmapped_returns_none() {
        assert!(translate_key(KeyCode::Enter).is_none());
        assert!(translate_key(KeyCode::Tab).is_none());
    }

    #[test]
    fn test_building_commands_exist() {
        let place = GameCommand::PlaceBuilding {
            building_type: 1,
            cell_x: 10,
            cell_y: 20,
            rotation: 0,
        };
        assert!(matches!(place, GameCommand::PlaceBuilding { .. }));

        let cancel = GameCommand::CancelPlacement;
        assert!(matches!(cancel, GameCommand::CancelPlacement));

        let build_mode = GameCommand::EnterBuildMode { building_type: 5 };
        assert!(matches!(build_mode, GameCommand::EnterBuildMode { .. }));
    }
}
