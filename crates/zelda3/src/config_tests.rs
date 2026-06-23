use super::*;

#[test]
fn bool_parser_matches_c_truth_table() {
    let mut out = false;
    assert!(parse_bool_str("true", Some(&mut out)));
    assert!(out);
    assert!(parse_bool_str("off", Some(&mut out)));
    assert!(!out);
    assert!(parse_bool_str("yes", None));
    assert!(!parse_bool_str("maybe", Some(&mut out)));
    assert!(!parse_bool_str("10", Some(&mut out)));
}

#[test]
fn integer_parser_accepts_c_numeric_prefixes() {
    assert_eq!(parse_i32("320abc"), 320);
    assert_eq!(parse_i32("  -12x"), -12);
    assert_eq!(parse_i32("+77junk"), 77);
    assert_eq!(parse_i32("x77"), 0);
}

#[test]
fn config_value_bytes_preserve_parser_byte_identity() {
    let value: String = [b'm', b's', b'u', b'/', 0x80, b'.', b'p', b'c', b'm']
        .into_iter()
        .map(char::from)
        .collect();
    assert_eq!(
        config_value_bytes(&value),
        vec![b'm', b's', b'u', b'/', 0x80, b'.', b'p', b'c', b'm']
    );
}

#[test]
fn default_keyboard_and_modified_lookup_match_c_shape() {
    let mut ctx = ConfigContext::default();
    ctx.parse_config_file(Some("__missing_config__"));

    assert_eq!(
        ctx.find_cmd_for_sdl_key(SDLK_UP, 0),
        KEY_COMMAND_CONTROLS as i32
    );
    assert_eq!(
        ctx.find_cmd_for_sdl_key(SDLK_RETURN, KMOD_ALT),
        KEY_COMMAND_FULLSCREEN as i32
    );
    assert_eq!(
        ctx.find_cmd_for_sdl_key(b'r' as SdlKeycode, KMOD_CTRL),
        KEY_COMMAND_RESET as i32
    );
    assert_eq!(
        ctx.find_cmd_for_sdl_key(SDLK_RSHIFT, KMOD_SHIFT),
        KEY_COMMAND_CONTROLS as i32 + 4
    );
}

#[test]
fn gamepad_modifier_entries_precede_less_specific_entries() {
    let mut ctx = ConfigContext::default();
    ctx.parse_gamepad_array("L1+A,A", KEY_COMMAND_CONTROLS, 2);
    assert_eq!(
        ctx.find_cmd_for_gamepad_button(GAMEPAD_BUTTON_A, 1 << GAMEPAD_BUTTON_L1),
        KEY_COMMAND_CONTROLS as i32
    );
    assert_eq!(
        ctx.find_cmd_for_gamepad_button(GAMEPAD_BUTTON_A, 0),
        KEY_COMMAND_CONTROLS as i32 + 1
    );
}

#[test]
fn sdl_key_name_resolver_matches_sdl_names() {
    assert_eq!(sdl_get_key_from_name("Return"), SDLK_RETURN);
    assert_eq!(sdl_get_key_from_name("Enter"), SDLK_UNKNOWN);
    assert_eq!(sdl_get_key_from_name("F24"), 1073741939);
    assert_eq!(sdl_get_key_from_name("Keypad Enter"), 1073741912);
    assert_eq!(sdl_get_key_from_name("Left Ctrl"), SDLK_LCTRL);
    assert_eq!(sdl_get_key_from_name("Right Shift"), SDLK_RSHIFT);
    assert_eq!(sdl_get_key_from_name("a"), b'a' as SdlKeycode);
    assert_eq!(sdl_get_key_from_name("A"), b'a' as SdlKeycode);
}

#[test]
fn config_file_parses_sections_and_includes() {
    let dir = std::env::temp_dir().join(format!("zelda3-rs-config-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let child = dir.join("child.ini");
    let root = dir.join("root.ini");
    fs::write(
        &child,
        "[Features]\nItemSwitchLR = true\n[GamepadMap]\nControls = L1+A,A\n",
    )
    .unwrap();
    fs::write(
        &root,
        "!include child.ini\n[Graphics]\nWindowSize=320x240\nOutputMethod=OpenGL ES\nShader=\n[Sound]\nEnableMSU=deluxe-opuz\nMSUVolume=77\n[General]\nExtendedAspectRatio=extend_y,16:9\nLanguage=en\n[KeyMap]\nReset=Ctrl+r\n",
    )
    .unwrap();

    let mut ctx = ConfigContext::default();
    ctx.parse_config_file(Some(root.to_str().unwrap()));
    assert_eq!(ctx.config.window_width, 320);
    assert_eq!(ctx.config.window_height, 240);
    assert_eq!(ctx.config.output_method, OUTPUT_METHOD_OPENGL_ES);
    assert_eq!(ctx.config.shader, None);
    assert_eq!(
        ctx.config.enable_msu,
        MSU_FEATURE_MSU_DELUXE | MSU_FEATURE_OPUZ
    );
    assert_eq!(ctx.config.msuvolume, 77);
    assert!(ctx.config.extend_y);
    assert_ne!(ctx.config.features0 & FEATURE_SWITCH_LR, 0);
    assert_eq!(ctx.config.language.as_deref(), Some("en"));
    assert_eq!(
        ctx.find_cmd_for_sdl_key(b'r' as SdlKeycode, KMOD_CTRL),
        KEY_COMMAND_RESET as i32
    );
    assert_eq!(
        ctx.find_cmd_for_gamepad_button(GAMEPAD_BUTTON_A, 1 << GAMEPAD_BUTTON_L1),
        KEY_COMMAND_CONTROLS as i32
    );

    fs::remove_dir_all(&dir).unwrap();
}
