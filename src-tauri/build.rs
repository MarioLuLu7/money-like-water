fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new().app_manifest(
            tauri_build::AppManifest::new().commands(&[
                "get_usage_snapshot",
                "get_usage_snapshot_for_source",
                "update_tray_tooltip",
                "position_meter_window",
                "toggle_meter_window",
                "get_meter_settings",
                "save_meter_settings",
                "save_meter_layout_settings",
                "get_chatgpt_access_token",
                "fetch_ai_member_auth_token",
                "set_ai_member_auth_token",
            ]),
        ),
    )
    .expect("failed to run Tauri build script");
}
