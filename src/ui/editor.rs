//! Editor window: toolbar | canvas | drawer.

use egui::{
    Align, Button, CentralPanel, Color32, ComboBox, Context, Frame, Layout, RichText, ScrollArea,
    Sense, SidePanel, Slider, Stroke, TextEdit, TopBottomPanel, Ui, Vec2,
};

use crate::color::{parse_color, to_hex, with_hex};
use crate::model::{
    AlignEdge, DrawerTab, KeyShape, PressEffect, VisualTheme, HOTKEY_OPEN_EDITOR,
    HOTKEY_TOGGLE_LOCK, HOTKEY_TOGGLE_VISIBILITY,
};
use crate::persist::{export_profile, import_profile};
use crate::platform;
use crate::state::{AppState, DragSession, ResizeHandle};
use crate::templates::{all_templates, theme_name};
use crate::ui::widgets::{key_rect, paint_key, paint_resize_handles};

pub fn show_editor(ctx: &Context, state: &mut AppState) {
    apply_theme(ctx);

    TopBottomPanel::top("title").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("KEY OVERLAY")
                    .strong()
                    .color(Color32::from_rgb(34, 211, 238))
                    .size(16.0),
            );
            ui.label(
                RichText::new(state.profile().name.clone())
                    .small()
                    .color(Color32::GRAY),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if let Some((msg, at)) = &state.status {
                    if at.elapsed().as_secs_f32() < 2.5 {
                        ui.label(RichText::new(msg).small().color(Color32::LIGHT_GREEN));
                    }
                }
            });
        });
    });

    SidePanel::left("toolbar")
        .exact_width(200.0)
        .show(ctx, |ui| {
            toolbar(ui, state);
        });

    if state.drawer.is_some() {
        SidePanel::right("drawer")
            .exact_width(300.0)
            .show(ctx, |ui| {
                drawer(ui, state);
            });
    }

    CentralPanel::default().show(ctx, |ui| {
        canvas(ui, state);
    });

    handle_editor_shortcuts(ctx, state);
}

fn apply_theme(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::from_rgb(8, 12, 20);
    visuals.window_fill = Color32::from_rgb(10, 16, 28);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(20, 30, 45);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(30, 45, 65);
    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(34, 211, 238, 60);
    style.visuals = visuals;
    ctx.set_style(style);
}

fn toolbar(ui: &mut Ui, state: &mut AppState) {
    ui.add_space(6.0);
    if ui
        .add_sized(
            [ui.available_width(), 36.0],
            Button::new(RichText::new("Place Overlay").strong()),
        )
        .clicked()
    {
        state.place_overlay();
    }

    if state.profile().show_kps_meter {
        ui.add_space(6.0);
        ui.label(
            RichText::new(format!("{} KPS", state.kps))
                .monospace()
                .color(Color32::from_rgb(34, 211, 238)),
        );
    }

    ui.separator();
    if ui.button("Add Custom Key").clicked() {
        state.add_key();
    }
    ui.menu_button("Mouse Click Zone", |ui| {
        if ui.button("Left click").clicked() {
            state.add_mouse_zone("left");
            ui.close_menu();
        }
        if ui.button("Right click").clicked() {
            state.add_mouse_zone("right");
            ui.close_menu();
        }
        if ui.button("Middle click").clicked() {
            state.add_mouse_zone("middle");
            ui.close_menu();
        }
    });
    if ui.button("Add Controller Pad").clicked() {
        state.add_controller_pad();
    }
    ui.menu_button("Add Joystick", |ui| {
        if ui.button("Left stick (LS)").clicked() {
            state.add_joystick("PadLS");
            ui.close_menu();
        }
        if ui.button("Right stick (RS)").clicked() {
            state.add_joystick("PadRS");
            ui.close_menu();
        }
    });

    ui.separator();
    drawer_btn(ui, state, DrawerTab::Visuals, "Customize");
    drawer_btn(ui, state, DrawerTab::Layouts, "Layouts");
    drawer_btn(ui, state, DrawerTab::Settings, "Settings");

    ui.separator();
    let snap = state.profile().snap_to_grid;
    if ui
        .selectable_label(snap, if snap { "Grid Snap: On" } else { "Grid Snap: Off" })
        .clicked()
    {
        state.profile_mut().snap_to_grid = !snap;
        state.dirty = true;
    }

    if !state.selected.is_empty() {
        if ui
            .button(RichText::new(format!("Delete ({})", state.selected.len())).color(Color32::LIGHT_RED))
            .clicked()
        {
            state.remove_selected();
        }
    }

    if state.selected.len() > 1 {
        ui.label(RichText::new("Align").small().color(Color32::GRAY));
        ui.horizontal_wrapped(|ui| {
            for (edge, label) in [
                (AlignEdge::Left, "L"),
                (AlignEdge::Center, "C"),
                (AlignEdge::Right, "R"),
                (AlignEdge::Top, "T"),
                (AlignEdge::Middle, "M"),
                (AlignEdge::Bottom, "B"),
            ] {
                if ui.small_button(label).clicked() {
                    state.align_selected(edge);
                }
            }
        });
    }

    ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
        ui.label(
            RichText::new(format!(
                "Ctrl+Z undo · {HOTKEY_TOGGLE_LOCK} lock · {HOTKEY_TOGGLE_VISIBILITY} hide · {HOTKEY_OPEN_EDITOR} editor"
            ))
            .small()
            .color(Color32::DARK_GRAY),
        );
    });
}

fn drawer_btn(ui: &mut Ui, state: &mut AppState, tab: DrawerTab, label: &str) {
    let active = state.drawer == Some(tab);
    if ui.selectable_label(active, label).clicked() {
        if active {
            state.drawer = None;
        } else {
            state.drawer = Some(tab);
            if matches!(tab, DrawerTab::Visuals) {
                // open visuals with themes/animations nearby via subtabs
            }
        }
    }
}

fn drawer(ui: &mut Ui, state: &mut AppState) {
    let Some(tab) = state.drawer else { return };
    ui.horizontal(|ui| {
        for (t, label) in [
            (DrawerTab::Visuals, "Visuals"),
            (DrawerTab::Themes, "Themes"),
            (DrawerTab::Animations, "Motion"),
            (DrawerTab::Layouts, "Layouts"),
            (DrawerTab::Settings, "Settings"),
        ] {
            if ui.selectable_label(state.drawer == Some(t), label).clicked() {
                state.drawer = Some(t);
            }
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.small_button("✕").clicked() {
                state.drawer = None;
            }
        });
    });
    ui.separator();
    ScrollArea::vertical().show(ui, |ui| match tab {
        DrawerTab::Visuals => visuals_tab(ui, state),
        DrawerTab::Themes => themes_tab(ui, state),
        DrawerTab::Animations => animations_tab(ui, state),
        DrawerTab::Layouts => layouts_tab(ui, state),
        DrawerTab::Settings => settings_tab(ui, state),
    });
}

fn visuals_tab(ui: &mut Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.label("Scope");
        if ui
            .selectable_label(!state.style_scope_all, "Selection")
            .clicked()
        {
            state.style_scope_all = false;
        }
        if ui.selectable_label(state.style_scope_all, "All keys").clicked() {
            state.style_scope_all = true;
        }
    });

    let reference = if !state.style_scope_all {
        state
            .profile()
            .keys
            .iter()
            .find(|k| state.selected.contains(&k.id))
            .cloned()
    } else {
        state.profile().keys.first().cloned()
    };

    let Some(reference) = reference else {
        ui.label("Select or add a key to edit styles.");
        return;
    };

    if !state.style_scope_all && state.selected.len() == 1 {
        ui.label(RichText::new("Key").strong());
        let id = reference.id.clone();
        let mut label = reference.label.clone();
        if ui.text_edit_singleline(&mut label).changed() {
            if let Some(k) = state.profile_mut().keys.iter_mut().find(|k| k.id == id) {
                k.label = label;
                state.dirty = true;
            }
        }
        ui.horizontal(|ui| {
            ui.monospace(&reference.code);
            let capturing = state.capturing.as_deref() == Some(id.as_str());
            if ui
                .button(if capturing { "Press…" } else { "Rebind" })
                .clicked()
            {
                if capturing {
                    state.capturing = None;
                } else {
                    state.capturing = Some(id);
                }
            }
        });
        ui.separator();
    }

    ui.label(RichText::new("Colours").strong());
    color_edit(ui, state, "Background", &reference.style.background_color, |s, v| {
        s.update_selected_style(|st| st.background_color = v.clone());
    });
    color_edit(ui, state, "Border", &reference.style.border_color, |s, v| {
        s.update_selected_style(|st| st.border_color = v.clone());
    });
    color_edit(ui, state, "Active glow", &reference.style.active_glow_color, |s, v| {
        s.update_selected_style(|st| st.active_glow_color = v.clone());
    });
    color_edit(ui, state, "Text", &reference.style.text_color, |s, v| {
        s.update_selected_style(|st| st.text_color = v.clone());
    });

    if !state.style_scope_all {
        ui.label("Outline");
        let mut shape = reference.shape;
        let shape_label = match shape {
            KeyShape::Rectangle => "Rectangle",
            KeyShape::Circle => "Circle",
            KeyShape::Stick => "Joystick",
        };
        ComboBox::from_id_salt("shape")
            .selected_text(shape_label)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut shape, KeyShape::Rectangle, "Rectangle");
                ui.selectable_value(&mut shape, KeyShape::Circle, "Circle");
                ui.selectable_value(&mut shape, KeyShape::Stick, "Joystick");
            });
        if shape != reference.shape {
            let sel = state.selected.clone();
            for k in state.profile_mut().keys.iter_mut() {
                if sel.contains(&k.id) {
                    k.shape = shape;
                }
            }
            state.dirty = true;
        }
    }

    let mut radius = reference.style.border_radius;
    if ui
        .add(Slider::new(&mut radius, 0.0..=48.0).text("Radius"))
        .changed()
    {
        state.update_selected_style(|st| st.border_radius = radius);
    }
    let mut opacity = reference.style.opacity;
    if ui
        .add(Slider::new(&mut opacity, 0.1..=1.0).text("Opacity"))
        .changed()
    {
        state.update_selected_style(|st| st.opacity = opacity);
    }
    let mut font = reference.style.font_size;
    if ui
        .add(Slider::new(&mut font, 8.0..=32.0).text("Font size"))
        .changed()
    {
        state.update_selected_style(|st| st.font_size = font);
    }

    ui.separator();
    let mut show_label = reference.style.show_label;
    if ui.checkbox(&mut show_label, "Key label").changed() {
        state.update_selected_style(|st| st.show_label = show_label);
    }
    let mut show_count = reference.style.show_press_count;
    if ui.checkbox(&mut show_count, "Press count").changed() {
        state.update_selected_style(|st| st.show_press_count = show_count);
    }
    let mut show_dur = reference.style.show_duration;
    if ui.checkbox(&mut show_dur, "Hold duration").changed() {
        state.update_selected_style(|st| st.show_duration = show_dur);
    }
}

fn color_edit(
    ui: &mut Ui,
    state: &mut AppState,
    label: &str,
    current: &str,
    apply: impl FnOnce(&mut AppState, String),
) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut rgba = parse_color(current).to_egui();
        if ui.color_edit_button_srgba(&mut rgba).changed() {
            let hex = format!("#{:02x}{:02x}{:02x}", rgba.r(), rgba.g(), rgba.b());
            apply(state, with_hex(current, &hex));
        }
        ui.monospace(to_hex(current));
    });
}

fn themes_tab(ui: &mut Ui, state: &mut AppState) {
    ui.label("Visual themes");
    for theme in [
        VisualTheme::Cyberpunk,
        VisualTheme::Glassmorphism,
        VisualTheme::RetroArcade,
        VisualTheme::StealthMinimal,
        VisualTheme::RgbWave,
    ] {
        let active = state.profile().global_theme == theme;
        if ui
            .selectable_label(active, theme_name(theme))
            .clicked()
        {
            state.apply_theme(theme);
        }
    }
}

fn animations_tab(ui: &mut Ui, state: &mut AppState) {
    ui.label("Press effect");
    let current = state
        .profile()
        .keys
        .iter()
        .find(|k| state.selected.contains(&k.id))
        .or_else(|| state.profile().keys.first())
        .map(|k| k.style.press_effect)
        .unwrap_or(PressEffect::Glow);

    for effect in [
        PressEffect::Glow,
        PressEffect::GlowPulse,
        PressEffect::KeyDrop,
        PressEffect::BorderRipple,
        PressEffect::None,
    ] {
        if ui
            .selectable_label(current == effect, AppState::effect_label(effect))
            .clicked()
        {
            state.update_selected_style(|st| st.press_effect = effect);
        }
    }
}

fn layouts_tab(ui: &mut Ui, state: &mut AppState) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Saved layouts").strong());
        if ui.button("+ New").clicked() {
            state.create_layout();
        }
    });

    let profiles: Vec<_> = state.library.profiles.iter().cloned().collect();
    let active = state.library.active_id.clone();
    for p in profiles {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                let selected = p.id == active;
                if ui.selectable_label(selected, &p.name).clicked() {
                    state.switch_layout(&p.id);
                }
                if ui.small_button("⧉").on_hover_text("Duplicate").clicked() {
                    state.duplicate_layout(Some(&p.id));
                }
                let can_delete = state.library.profiles.len() > 1;
                if ui
                    .add_enabled(can_delete, Button::new("Del").small())
                    .on_hover_text("Delete")
                    .clicked()
                {
                    state.delete_layout(&p.id);
                }
            });
            let mut name = p.name.clone();
            if ui
                .add(TextEdit::singleline(&mut name).desired_width(240.0).hint_text("Rename"))
                .changed()
            {
                state.rename_layout(&p.id, name);
            }
            ui.label(
                RichText::new(format!("{} keys", p.keys.len()))
                    .small()
                    .color(Color32::GRAY),
            );
        });
    }

    ui.add_space(12.0);
    ui.label(RichText::new("Presets").strong());
    for t in all_templates() {
        let active = state.profile().template_id == t.id;
        if ui
            .selectable_label(active, format!("{} — {}", t.name, t.description))
            .clicked()
        {
            state.load_template(t.id);
        }
    }
}

fn settings_tab(ui: &mut Ui, state: &mut AppState) {
    let mut show_kps = state.profile().show_kps_meter;
    if ui.checkbox(&mut show_kps, "Show KPS meter").changed() {
        state.profile_mut().show_kps_meter = show_kps;
        state.dirty = true;
    }
    let mut opacity = state.profile().window_opacity;
    if ui
        .add(Slider::new(&mut opacity, 0.1..=1.0).text("Overlay opacity"))
        .changed()
    {
        state.profile_mut().window_opacity = opacity;
        state.dirty = true;
    }
    let mut grid = state.profile().grid_size;
    if ui
        .add(Slider::new(&mut grid, 2.0..=40.0).text("Grid size"))
        .changed()
    {
        state.profile_mut().grid_size = grid.round();
        state.dirty = true;
    }

    ui.separator();
    ui.label(RichText::new("Target app filter").strong());
    let mut enabled = state.profile().target_app_enabled;
    if ui.checkbox(&mut enabled, "Only show while app matches").changed() {
        state.profile_mut().target_app_enabled = enabled;
        platform::set_filter(enabled, state.profile().target_app_match.clone());
        state.dirty = true;
    }
    let mut match_text = state.profile().target_app_match.clone();
    if ui.text_edit_singleline(&mut match_text).changed() {
        state.profile_mut().target_app_match = match_text.clone();
        platform::set_filter(state.profile().target_app_enabled, match_text);
        state.dirty = true;
    }
    if ui.button("Use currently focused app").clicked() {
        if let Ok(fg) = platform::get_foreground_app() {
            let name = if fg.process_name.is_empty() {
                fg.window_title
            } else {
                fg.process_name
            };
            state.profile_mut().target_app_match = name.clone();
            state.profile_mut().target_app_enabled = true;
            platform::set_filter(true, name);
            state.dirty = true;
        }
    }

    ui.separator();
    if ui.button("Export profile…").clicked() {
        let profile = state.profile().clone();
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name(format!("{}.json", profile.name.replace(['/', '\\'], "-")))
            .add_filter("Key Overlay profile", &["json"])
            .save_file()
        {
            match export_profile(&path, &profile) {
                Ok(()) => state.flash("Exported"),
                Err(e) => state.flash(format!("Export failed: {e}")),
            }
        }
    }
    if ui.button("Import profile…").clicked() {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Key Overlay profile", &["json"])
            .pick_file()
        {
            match import_profile(&path) {
                Ok(mut p) => {
                    p.id = format!("profile-{}", uuid::Uuid::new_v4());
                    state.library.active_id = p.id.clone();
                    state.library.profiles.push(p);
                    state.past.clear();
                    state.future.clear();
                    state.dirty = true;
                    state.flash("Imported");
                }
                Err(e) => state.flash(format!("Import failed: {e}")),
            }
        }
    }

    ui.separator();
    ui.label(
        RichText::new(format!(
            "Hotkeys: {HOTKEY_TOGGLE_VISIBILITY} visibility · {HOTKEY_TOGGLE_LOCK} lock"
        ))
        .small()
        .color(Color32::GRAY),
    );
}

fn canvas(ui: &mut Ui, state: &mut AppState) {
    Frame::canvas(ui.style())
        .fill(Color32::from_rgb(6, 10, 18))
        .stroke(Stroke::new(1.0_f32, Color32::from_white_alpha(10)))
        .show(ui, |ui| {
            let (response, painter) =
                ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
            state.canvas_size = response.rect.size();

            if state.profile().snap_to_grid {
                let g = state.profile().grid_size.max(2.0);
                let mut x = response.rect.left();
                while x < response.rect.right() {
                    painter.line_segment(
                        [
                            egui::pos2(x, response.rect.top()),
                            egui::pos2(x, response.rect.bottom()),
                        ],
                        Stroke::new(1.0_f32, Color32::from_white_alpha(8)),
                    );
                    x += g;
                }
                let mut y = response.rect.top();
                while y < response.rect.bottom() {
                    painter.line_segment(
                        [
                            egui::pos2(response.rect.left(), y),
                            egui::pos2(response.rect.right(), y),
                        ],
                        Stroke::new(1.0_f32, Color32::from_white_alpha(8)),
                    );
                    y += g;
                }
            }

            if state.capturing.is_some() {
                painter.text(
                    response.rect.center_top() + Vec2::new(0.0, 24.0),
                    egui::Align2::CENTER_TOP,
                    "Press a key, mouse button, or controller… (Esc to cancel)",
                    egui::FontId::proportional(14.0),
                    Color32::from_rgb(251, 191, 36),
                );
            }

            // Shift origin so keys are in canvas-local coords.
            let origin = response.rect.min;
            let keys = state.profile().keys.clone();
            let mut hit_id = None;
            let mut resize_hit: Option<(String, ResizeHandle)> = None;

            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(response.rect), |ui| {
                ui.set_clip_rect(response.rect);
                // Translate painting into canvas space by modifying key positions temporarily.
                for key in &keys {
                    let mut k = key.clone();
                    k.x += origin.x;
                    k.y += origin.y;
                    let resp = paint_key(ui, state, &k, true);
                    if state.selected.contains(&key.id) {
                        paint_resize_handles(ui, key_rect(&k));
                        let r = key_rect(&k);
                        let hs = 10.0;
                        let corners = [
                            (ResizeHandle::Nw, r.left_top()),
                            (ResizeHandle::Ne, r.right_top()),
                            (ResizeHandle::Sw, r.left_bottom()),
                            (ResizeHandle::Se, r.right_bottom()),
                        ];
                        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                            for (h, c) in corners {
                                if c.distance(pos) <= hs {
                                    resize_hit = Some((key.id.clone(), h));
                                }
                            }
                        }
                    }
                    if resp.clicked() || resp.drag_started() {
                        hit_id = Some(key.id.clone());
                    }
                }
            });

            let pointer = ui.input(|i| i.pointer.interact_pos());
            let primary_down = ui.input(|i| i.pointer.primary_pressed());
            let primary_released = ui.input(|i| i.pointer.primary_released());
            let dragging = ui.input(|i| i.pointer.is_decidedly_dragging());
            let shift = ui.input(|i| i.modifiers.shift);

            if primary_down {
                if let Some((id, handle)) = resize_hit.clone() {
                    let origin = state
                        .profile()
                        .keys
                        .iter()
                        .find(|k| k.id == id)
                        .map(key_rect);
                    if let Some(origin) = origin {
                        state.push_history();
                        state.drag = Some(DragSession::Resize {
                            id,
                            handle,
                            start: pointer.unwrap_or_default(),
                            origin,
                        });
                    }
                } else if let Some(id) = hit_id {
                    if shift {
                        if state.selected.contains(&id) {
                            state.selected.remove(&id);
                        } else {
                            state.selected.insert(id);
                        }
                    } else if !state.selected.contains(&id) {
                        state.selected = std::collections::HashSet::from([id]);
                    }
                    let origins = state
                        .profile()
                        .keys
                        .iter()
                        .filter(|k| state.selected.contains(&k.id))
                        .map(|k| (k.id.clone(), egui::pos2(k.x, k.y)))
                        .collect();
                    state.push_history();
                    state.drag = Some(DragSession::Move {
                        start: pointer.unwrap_or_default(),
                        origins,
                    });
                } else {
                    state.selected.clear();
                    state.drag = None;
                }
            }

            if dragging {
                if let (Some(pos), Some(session)) = (pointer, state.drag.as_ref()) {
                    match session {
                        DragSession::Move { start, origins } => {
                            let mut dx = pos.x - start.x;
                            let mut dy = pos.y - start.y;
                            if state.profile().snap_to_grid {
                                let g = state.profile().grid_size;
                                dx = (dx / g).round() * g;
                                dy = (dy / g).round() * g;
                            }
                            let origins = origins.clone();
                            for (id, origin) in origins {
                                if let Some(k) = state.profile_mut().keys.iter_mut().find(|k| k.id == id)
                                {
                                    k.x = (origin.x + dx).max(0.0);
                                    k.y = (origin.y + dy).max(0.0);
                                }
                            }
                        }
                        DragSession::Resize {
                            id,
                            handle,
                            start,
                            origin,
                        } => {
                            let dx = pos.x - start.x;
                            let dy = pos.y - start.y;
                            let id = id.clone();
                            let handle = *handle;
                            let origin = *origin;
                            if let Some(k) = state.profile_mut().keys.iter_mut().find(|k| k.id == id) {
                                let mut x = origin.min.x;
                                let mut y = origin.min.y;
                                let mut w = origin.width();
                                let mut h = origin.height();
                                match handle {
                                    ResizeHandle::Se => {
                                        w = (w + dx).max(20.0);
                                        h = (h + dy).max(20.0);
                                    }
                                    ResizeHandle::Sw => {
                                        w = (w - dx).max(20.0);
                                        h = (h + dy).max(20.0);
                                        x = origin.max.x - w;
                                    }
                                    ResizeHandle::Ne => {
                                        w = (w + dx).max(20.0);
                                        h = (h - dy).max(20.0);
                                        y = origin.max.y - h;
                                    }
                                    ResizeHandle::Nw => {
                                        w = (w - dx).max(20.0);
                                        h = (h - dy).max(20.0);
                                        x = origin.max.x - w;
                                        y = origin.max.y - h;
                                    }
                                }
                                k.x = x;
                                k.y = y;
                                k.width = w;
                                k.height = h;
                            }
                        }
                    }
                }
            }

            if primary_released {
                if state.drag.is_some() {
                    state.touch_profile();
                }
                state.drag = None;
            }
        });
}

fn handle_editor_shortcuts(ctx: &Context, state: &mut AppState) {
    ctx.input(|i| {
        if i.key_pressed(egui::Key::Escape) {
            if state.capturing.is_some() {
                state.capturing = None;
            } else {
                state.selected.clear();
            }
        }
    });

    if state.shortcuts_blocked() {
        return;
    }

    let mut undo = false;
    let mut redo = false;
    let mut delete = false;
    let mut nudge = None;
    ctx.input(|i| {
        let mod_key = i.modifiers.command;
        if mod_key && i.key_pressed(egui::Key::Z) {
            if i.modifiers.shift {
                redo = true;
            } else {
                undo = true;
            }
        }
        if mod_key && i.key_pressed(egui::Key::Y) {
            redo = true;
        }
        if i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace) {
            delete = true;
        }
        let step = if i.modifiers.shift {
            10.0
        } else if state.profile().snap_to_grid {
            state.profile().grid_size
        } else {
            1.0
        };
        if i.key_pressed(egui::Key::ArrowLeft) {
            nudge = Some((-step, 0.0));
        }
        if i.key_pressed(egui::Key::ArrowRight) {
            nudge = Some((step, 0.0));
        }
        if i.key_pressed(egui::Key::ArrowUp) {
            nudge = Some((0.0, -step));
        }
        if i.key_pressed(egui::Key::ArrowDown) {
            nudge = Some((0.0, step));
        }
    });

    if undo {
        state.undo();
    }
    if redo {
        state.redo();
    }
    if delete {
        state.remove_selected();
    }
    if let Some((dx, dy)) = nudge {
        state.nudge_selected(dx, dy);
    }
}
