use eframe::egui::{
    self, Align, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Frame, Layout,
    Margin, RichText, Stroke, TextEdit, Vec2,
};
use rfd::FileDialog;
use security_service::{
    decrypt_file_command, embed_file_command, encrypt_file_command, extract_file_command,
    generate_derived_password_command, DecryptRequest, EmbedRequest, EncryptRequest,
    EncryptionAlgorithm, ExtractRequest, PasswordGeneratorRequest, StegoMode, VaultError,
};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

const APP_ICON: &[u8] = include_bytes!("../assets/logo.png");

const BG: Color32 = Color32::from_rgb(239, 244, 245);
const SHELL: Color32 = Color32::from_rgb(228, 238, 241);
const CARD: Color32 = Color32::from_rgb(255, 255, 255);
const BORDER: Color32 = Color32::from_rgb(217, 229, 232);
const INPUT: Color32 = Color32::from_rgb(248, 251, 251);
const PRIMARY: Color32 = Color32::from_rgb(20, 93, 111);
const PRIMARY_HOVER: Color32 = Color32::from_rgb(28, 108, 126);
const MINT: Color32 = Color32::from_rgb(216, 238, 243);
const TEXT: Color32 = Color32::from_rgb(22, 58, 67);
const MUTED: Color32 = Color32::from_rgb(91, 114, 121);
const PAGE_GAP: f32 = 6.0;
const CONTROL_HEIGHT: f32 = 26.0;
const ACTION_HEIGHT: f32 = 30.0;
const LABEL_WIDTH: f32 = 48.0;
const FORM_BLOCK_HEIGHT: f32 = 116.0;
const COMPACT_BLOCK_HEIGHT: f32 = 92.0;
const ACTION_WIDTH: f32 = 220.0;
const CRYPTO_HEADER: &[u8] = b"SECURE_ENC_V5";

pub fn run_app() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("VaultCanvas")
            .with_inner_size([560.0, 420.0])
            .with_min_inner_size([560.0, 420.0])
            .with_icon(load_app_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "VaultCanvas",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_theme(egui::Theme::Light);
            configure_fonts(&cc.egui_ctx);
            configure_style(&cc.egui_ctx);
            Ok(Box::<VaultCanvasApp>::default())
        }),
    )
}

fn load_app_icon() -> Arc<egui::IconData> {
    Arc::new(eframe::icon_data::from_png_bytes(APP_ICON).unwrap_or_default())
}

fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    if let Some(data) = load_zh_font() {
        let font_name = "zh_cn".to_string();
        fonts
            .font_data
            .insert(font_name.clone(), FontData::from_owned(data).into());
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, font_name.clone());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, font_name);
    }
    ctx.set_fonts(fonts);
}

fn load_zh_font() -> Option<Vec<u8>> {
    for path in candidate_font_paths() {
        if let Ok(bytes) = fs::read(path) {
            return Some(bytes);
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn candidate_font_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(windir) = std::env::var_os("WINDIR") {
        let font_dir = PathBuf::from(windir).join("Fonts");
        for name in ["msyh.ttc", "msyhbd.ttc", "simhei.ttf", "simsun.ttc"] {
            paths.push(font_dir.join(name));
        }
    }
    paths
}

#[cfg(target_os = "macos")]
fn candidate_font_paths() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/System/Library/Fonts/PingFang.ttc"),
        PathBuf::from("/System/Library/Fonts/STHeiti Light.ttc"),
        PathBuf::from("/System/Library/Fonts/STHeiti Medium.ttc"),
    ]
}

#[cfg(all(unix, not(target_os = "macos")))]
fn candidate_font_paths() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"),
        PathBuf::from("/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc"),
        PathBuf::from("/usr/share/fonts/truetype/wqy/wqy-microhei.ttc"),
    ]
}

#[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
fn candidate_font_paths() -> Vec<PathBuf> {
    Vec::new()
}

fn configure_style(ctx: &egui::Context) {
    let mut style = egui::Style {
        visuals: egui::Visuals::light(),
        ..Default::default()
    };
    style.spacing.item_spacing = Vec2::new(6.0, 6.0);
    style.spacing.button_padding = Vec2::new(10.0, 5.0);
    style.spacing.interact_size = Vec2::new(28.0, 28.0);
    style.spacing.window_margin = Margin::same(6);
    style.spacing.indent = 0.0;
    style.visuals.panel_fill = BG;
    style.visuals.override_text_color = Some(TEXT);
    style.visuals.widgets.noninteractive.bg_fill = CARD;
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    style.visuals.widgets.inactive.bg_fill = INPUT;
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(242, 247, 248);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(197, 219, 224));
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(238, 245, 247);
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(181, 210, 216));
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, TEXT);
    style.visuals.widgets.open.bg_fill = Color32::from_rgb(242, 247, 248);
    style.visuals.widgets.open.bg_stroke = Stroke::new(1.0, Color32::from_rgb(197, 219, 224));
    style.visuals.widgets.open.fg_stroke = Stroke::new(1.0, TEXT);
    style.visuals.selection.bg_fill = PRIMARY;
    style.visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(248, 252, 252));
    style.visuals.extreme_bg_color = INPUT;
    style.visuals.faint_bg_color = Color32::from_rgb(244, 248, 249);
    style.visuals.window_fill = CARD;
    style.visuals.window_stroke = Stroke::new(1.0, BORDER);
    style.visuals.window_corner_radius = CornerRadius::same(12);
    style.visuals.menu_corner_radius = CornerRadius::same(12);
    style.visuals.widgets.inactive.corner_radius = CornerRadius::same(7);
    style.visuals.widgets.hovered.corner_radius = CornerRadius::same(7);
    style.visuals.widgets.active.corner_radius = CornerRadius::same(7);
    style.text_styles = [
        (egui::TextStyle::Heading, egui::FontId::proportional(15.5)),
        (egui::TextStyle::Body, egui::FontId::proportional(12.0)),
        (egui::TextStyle::Button, egui::FontId::proportional(11.0)),
        (egui::TextStyle::Small, egui::FontId::proportional(9.0)),
    ]
    .into();
    ctx.set_style(style);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Password,
    Crypto,
    Stego,
}

impl Tab {
    fn label(self) -> &'static str {
        match self {
            Tab::Password => "密码",
            Tab::Crypto => "加解密",
            Tab::Stego => "隐写",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Tab::Password => "密码生成",
            Tab::Crypto => "文件加解密",
            Tab::Stego => "文件隐写",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CryptoMode {
    Encrypt,
    Decrypt,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StegoModeUi {
    Embed,
    Extract,
}

#[derive(Default)]
struct PasswordState {
    main_password: String,
    id_password: String,
    output: String,
}

struct CryptoState {
    mode: CryptoMode,
    input_path: String,
    password: String,
    id_password: String,
}

impl Default for CryptoState {
    fn default() -> Self {
        Self {
            mode: CryptoMode::Encrypt,
            input_path: String::new(),
            password: String::new(),
            id_password: String::new(),
        }
    }
}

struct StegoState {
    mode: StegoModeUi,
    carrier_path: String,
    payload_path: String,
    password: String,
}

impl Default for StegoState {
    fn default() -> Self {
        Self {
            mode: StegoModeUi::Embed,
            carrier_path: String::new(),
            payload_path: String::new(),
            password: String::new(),
        }
    }
}

struct VaultCanvasApp {
    tab: Tab,
    password: PasswordState,
    crypto: CryptoState,
    stego: StegoState,
    status: String,
}

impl Default for VaultCanvasApp {
    fn default() -> Self {
        Self {
            tab: Tab::Password,
            password: PasswordState::default(),
            crypto: CryptoState::default(),
            stego: StegoState::default(),
            status: String::new(),
        }
    }
}

impl eframe::App for VaultCanvasApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(BG).inner_margin(Margin::same(5)))
            .show(ctx, |ui| {
                draw_top_bar(ui, self.tab, |tab| self.tab = tab);
                ui.add_space(5.0);

                let available = ui.available_size();
                let status = self.status.clone();
                ui.allocate_ui_with_layout(
                    Vec2::new(available.x, available.y),
                    Layout::top_down(Align::Min),
                    |ui| {
                        draw_panel(ui, self.tab.title(), status.as_str(), |ui| match self.tab {
                            Tab::Password => self.render_password(ui),
                            Tab::Crypto => self.render_crypto(ui),
                            Tab::Stego => self.render_stego(ui),
                        });
                    },
                );
            });
    }
}

impl VaultCanvasApp {
    fn render_password(&mut self, ui: &mut egui::Ui) {
        let wide = ui.available_width() >= 860.0;
        let content_width = if wide {
            ui.available_width().min(940.0)
        } else {
            ui.available_width().min(520.0)
        };

        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.set_max_width(content_width);
            ui.add_space(2.0);

            if wide {
                section_block_sized(ui, "凭据", 104.0, |ui| {
                    ui.columns(2, |columns| {
                        draw_secret_row(
                            &mut columns[0],
                            "主密码",
                            &mut self.password.main_password,
                        );
                        draw_secret_row(&mut columns[1], "ID 密码", &mut self.password.id_password);
                    });
                });

                ui.add_space(PAGE_GAP);
                section_block_sized(ui, "结果", 82.0, |ui| {
                    ui.label(
                        RichText::new("固定生成 16 位，含大小写字母、数字、特殊字符")
                            .size(8.6)
                            .color(MUTED),
                    );
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            Vec2::new((ui.available_width() - 310.0).max(260.0), CONTROL_HEIGHT),
                            TextEdit::singleline(&mut self.password.output)
                                .desired_width(f32::INFINITY)
                                .hint_text("生成后显示"),
                        );
                        ui.add_space(8.0);
                        if draw_primary_button(ui, "生成密码", Some(190.0)) {
                            self.run_password_generation();
                        }
                        if draw_secondary_button(ui, "复制", 82.0) {
                            ui.ctx().copy_text(self.password.output.clone());
                            self.status = "密码已复制".into();
                        }
                    });
                });
            } else {
                section_block(ui, "凭据", |ui| {
                    draw_secret_row(ui, "主密码", &mut self.password.main_password);
                    ui.add_space(PAGE_GAP - 1.0);
                    draw_secret_row(ui, "ID 密码", &mut self.password.id_password);
                });

                ui.add_space(PAGE_GAP);
                section_block(ui, "结果", |ui| {
                    ui.label(
                        RichText::new("固定生成 16 位，含大小写字母、数字、特殊字符")
                            .size(8.6)
                            .color(MUTED),
                    );
                    ui.add_space(4.0);
                    ui.add_sized(
                        Vec2::new(ui.available_width(), CONTROL_HEIGHT),
                        TextEdit::singleline(&mut self.password.output)
                            .desired_width(f32::INFINITY)
                            .hint_text("生成后显示"),
                    );
                });

                ui.add_space(PAGE_GAP);
                ui.horizontal_centered(|ui| {
                    if draw_primary_button(ui, "生成密码", Some((content_width - 90.0).max(172.0)))
                    {
                        self.run_password_generation();
                    }
                    if draw_secondary_button(ui, "复制", 82.0) {
                        ui.ctx().copy_text(self.password.output.clone());
                        self.status = "密码已复制".into();
                    }
                });
            }

            ui.add_space(2.0);
        });
    }

    fn run_password_generation(&mut self) {
        match generate_derived_password_command(PasswordGeneratorRequest {
            main_password: self.password.main_password.clone(),
            id_password: self.password.id_password.clone(),
            length: 16,
            use_lowercase: true,
            use_uppercase: true,
            use_digits: true,
            use_symbols: true,
        }) {
            Ok(value) => {
                self.password.output = value;
                self.status = "密码已生成".into();
            }
            Err(err) => self.status = friendly_error(&err),
        }
    }

    fn render_crypto(&mut self, ui: &mut egui::Ui) {
        let wide = ui.available_width() >= 860.0;
        let content_width = if wide {
            ui.available_width().min(940.0)
        } else {
            ui.available_width().min(494.0)
        };

        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.set_max_width(content_width);
            ui.add_space(2.0);
            draw_mode_switch(
                ui,
                self.crypto.mode == CryptoMode::Encrypt,
                "文件加密",
                "文件解密",
                |left| {
                    self.crypto.mode = if left {
                        CryptoMode::Encrypt
                    } else {
                        CryptoMode::Decrypt
                    }
                },
            );
            ui.add_space(5.0);

            if wide {
                let left_width = (content_width * 0.56).max(320.0);
                let right_width = (content_width - left_width - 8.0).max(240.0);
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    ui.set_width(content_width);
                    ui.allocate_ui(Vec2::new(left_width, 0.0), |ui| {
                        section_block_sized(ui, "文件", FORM_BLOCK_HEIGHT, |ui| {
                            draw_path_row(
                                ui,
                                if self.crypto.mode == CryptoMode::Encrypt {
                                    "源文件"
                                } else {
                                    "加密文件"
                                },
                                &mut self.crypto.input_path,
                            );
                        });
                    });
                    ui.allocate_ui(Vec2::new(right_width, 0.0), |ui| {
                        section_block_sized(ui, "密码", FORM_BLOCK_HEIGHT, |ui| {
                            draw_password_field(ui, "主密码", &mut self.crypto.password);
                            ui.add_space(PAGE_GAP - 1.0);
                            draw_password_field(ui, "ID 密码", &mut self.crypto.id_password);
                        });
                    });
                });
            } else {
                section_block(ui, "文件", |ui| {
                    draw_path_row(
                        ui,
                        if self.crypto.mode == CryptoMode::Encrypt {
                            "源文件"
                        } else {
                            "加密文件"
                        },
                        &mut self.crypto.input_path,
                    );
                });
                ui.add_space(PAGE_GAP);
                section_block(ui, "密码", |ui| {
                    draw_password_field(ui, "主密码", &mut self.crypto.password);
                    ui.add_space(PAGE_GAP - 1.0);
                    draw_password_field(ui, "ID 密码", &mut self.crypto.id_password);
                });
            }

            ui.add_space(PAGE_GAP);
            if draw_primary_button(
                ui,
                if self.crypto.mode == CryptoMode::Encrypt {
                    "执行加密"
                } else {
                    "执行解密"
                },
                Some(if wide { ACTION_WIDTH } else { content_width }),
            ) {
                self.run_crypto();
            }
        });
    }

    fn render_stego(&mut self, ui: &mut egui::Ui) {
        let wide = ui.available_width() >= 860.0;
        let content_width = if wide {
            ui.available_width().min(940.0)
        } else {
            ui.available_width().min(494.0)
        };

        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.set_max_width(content_width);
            ui.add_space(2.0);
            draw_mode_switch(
                ui,
                self.stego.mode == StegoModeUi::Embed,
                "文件隐写",
                "隐写回显",
                |left| {
                    self.stego.mode = if left {
                        StegoModeUi::Embed
                    } else {
                        StegoModeUi::Extract
                    }
                },
            );
            ui.add_space(5.0);

            if wide {
                let left_width = (content_width * 0.56).max(320.0);
                let right_width = (content_width - left_width - 8.0).max(240.0);
                let block_height = if self.stego.mode == StegoModeUi::Embed {
                    FORM_BLOCK_HEIGHT
                } else {
                    COMPACT_BLOCK_HEIGHT
                };
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    ui.set_width(content_width);
                    ui.allocate_ui(Vec2::new(left_width, 0.0), |ui| {
                        section_block_sized(ui, "文件", block_height, |ui| {
                            draw_path_row(ui, "载体文件", &mut self.stego.carrier_path);
                            if self.stego.mode == StegoModeUi::Embed {
                                ui.add_space(PAGE_GAP - 1.0);
                                draw_path_row(ui, "隐藏文件", &mut self.stego.payload_path);
                            }
                        });
                    });
                    ui.allocate_ui(Vec2::new(right_width, 0.0), |ui| {
                        section_block_sized(ui, "密码", block_height, |ui| {
                            draw_password_field(ui, "隐写密码", &mut self.stego.password);
                        });
                    });
                });
            } else {
                section_block(ui, "文件", |ui| {
                    draw_path_row(ui, "载体文件", &mut self.stego.carrier_path);
                    if self.stego.mode == StegoModeUi::Embed {
                        ui.add_space(PAGE_GAP - 1.0);
                        draw_path_row(ui, "隐藏文件", &mut self.stego.payload_path);
                    }
                });
                ui.add_space(PAGE_GAP);
                section_block(ui, "密码", |ui| {
                    draw_password_field(ui, "隐写密码", &mut self.stego.password);
                });
            }

            ui.add_space(PAGE_GAP);
            if draw_primary_button(
                ui,
                if self.stego.mode == StegoModeUi::Embed {
                    "执行隐写"
                } else {
                    "执行回显"
                },
                Some(if wide { ACTION_WIDTH } else { content_width }),
            ) {
                self.run_stego();
            }
        });
    }
    fn run_crypto(&mut self) {
        if let Err(message) = validate_crypto_inputs(&self.crypto) {
            self.status = message.into();
            return;
        }

        let Some(output_dir) = FileDialog::new().pick_folder() else {
            self.status = "已取消保存位置选择".into();
            return;
        };
        let (output_path, renamed) = ensure_available_output_path(build_crypto_output(
            output_dir.display().to_string().as_str(),
            &self.crypto.input_path,
            self.crypto.mode,
        ));
        let result = match self.crypto.mode {
            CryptoMode::Encrypt => encrypt_file_command(EncryptRequest {
                input_path: self.crypto.input_path.clone(),
                output_path,
                password: self.crypto.password.clone(),
                id_password: self.crypto.id_password.clone(),
                algorithm: EncryptionAlgorithm::Aes256Gcm,
            }),
            CryptoMode::Decrypt => decrypt_file_command(DecryptRequest {
                input_path: self.crypto.input_path.clone(),
                output_path,
                password: self.crypto.password.clone(),
                id_password: self.crypto.id_password.clone(),
            }),
        };
        self.status = match result {
            Ok(done) => success_message(&done.output_path, renamed),
            Err(err) => friendly_error(&err),
        };
    }

    fn run_stego(&mut self) {
        if let Err(message) = validate_stego_inputs(&self.stego) {
            self.status = message.into();
            return;
        }

        let Some(output_dir) = FileDialog::new().pick_folder() else {
            self.status = "已取消保存位置选择".into();
            return;
        };
        let (output_path, renamed) = ensure_available_output_path(build_stego_output(
            output_dir.display().to_string().as_str(),
            &self.stego.carrier_path,
            self.stego.mode,
        ));
        let result = match self.stego.mode {
            StegoModeUi::Embed => embed_file_command(EmbedRequest {
                carrier_path: self.stego.carrier_path.clone(),
                payload_path: self.stego.payload_path.clone(),
                output_path,
                password: self.stego.password.clone(),
                mode: StegoMode::Append,
            }),
            StegoModeUi::Extract => extract_file_command(ExtractRequest {
                carrier_path: self.stego.carrier_path.clone(),
                output_path,
                password: self.stego.password.clone(),
                mode: StegoMode::Append,
            }),
        };
        self.status = match result {
            Ok(done) => success_message(&done.output_path, renamed),
            Err(err) => friendly_error(&err),
        };
    }
}

fn draw_top_bar(ui: &mut egui::Ui, current: Tab, mut on_select: impl FnMut(Tab)) {
    Frame::new()
        .fill(SHELL)
        .stroke(Stroke::new(1.0, Color32::from_rgb(210, 225, 229)))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::symmetric(10, 5))
        .show(ui, |ui| {
            ui.set_height(38.0);
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    Vec2::new(148.0, 24.0),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.horizontal(|ui| {
                            brand_mark(ui);
                            ui.add_space(2.0);
                            ui.label(
                                RichText::new("VaultCanvas")
                                    .size(11.6)
                                    .strong()
                                    .color(Color32::from_rgb(27, 66, 77)),
                            );
                        });
                    },
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 5.0;
                    for tab in [Tab::Stego, Tab::Crypto, Tab::Password] {
                        let selected = current == tab;
                        let fill = if selected {
                            Color32::from_rgb(27, 104, 122)
                        } else {
                            Color32::from_rgb(244, 249, 250)
                        };
                        let text = if selected {
                            Color32::from_rgb(248, 252, 252)
                        } else {
                            Color32::from_rgb(39, 78, 88)
                        };
                        let stroke = if selected {
                            Stroke::new(1.0, Color32::from_rgb(23, 91, 107))
                        } else {
                            Stroke::new(1.0, Color32::from_rgb(205, 222, 226))
                        };
                        let button = egui::Button::new(
                            RichText::new(tab.label()).size(9.4).strong().color(text),
                        )
                        .fill(fill)
                        .stroke(stroke)
                        .corner_radius(CornerRadius::same(8))
                        .min_size(Vec2::new(60.0, 28.0));
                        if ui.add(button).clicked() {
                            on_select(tab);
                        }
                    }
                });
            });
        });
}

fn brand_mark(ui: &mut egui::Ui) {
    Frame::new()
        .fill(Color32::from_rgb(49, 111, 125))
        .stroke(Stroke::new(1.0, Color32::from_rgb(41, 95, 108)))
        .corner_radius(CornerRadius::same(7))
        .inner_margin(Margin::symmetric(5, 2))
        .show(ui, |ui| {
            ui.label(
                RichText::new("VC")
                    .size(8.6)
                    .strong()
                    .color(Color32::from_rgb(238, 246, 247)),
            );
        });
}

fn draw_panel(
    ui: &mut egui::Ui,
    title: &str,
    status: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.set_min_height(ui.available_height());
            ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(title)
                        .size(12.0)
                        .strong()
                        .color(Color32::from_rgb(23, 60, 69)),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if !status.is_empty() {
                        info_badge(ui, status);
                    }
                });
            });
            ui.add_space(5.0);
            add_contents(ui);
        });
}

fn section_block(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    Frame::new()
        .fill(Color32::from_rgb(249, 251, 251))
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(7))
        .show(ui, |ui| {
            ui.label(RichText::new(title).size(8.6).strong().color(MUTED));
            ui.add_space(3.0);
            add_contents(ui);
        });
}

fn section_block_sized(
    ui: &mut egui::Ui,
    title: &str,
    min_height: f32,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    Frame::new()
        .fill(Color32::from_rgb(249, 251, 251))
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(7))
        .show(ui, |ui| {
            ui.set_min_height(min_height);
            ui.label(RichText::new(title).size(8.6).strong().color(MUTED));
            ui.add_space(3.0);
            add_contents(ui);
        });
}

fn info_badge(ui: &mut egui::Ui, text: &str) {
    Frame::new()
        .fill(Color32::from_rgb(232, 241, 243))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(7, 3))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(8.0).color(MUTED));
        });
}

fn draw_mode_switch(
    ui: &mut egui::Ui,
    left_selected: bool,
    left: &str,
    right: &str,
    mut on_change: impl FnMut(bool),
) {
    Frame::new()
        .fill(Color32::from_rgb(244, 248, 249))
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(3))
        .show(ui, |ui| {
            ui.set_width(168.0);
            ui.horizontal(|ui| {
                for (is_left, label) in [(true, left), (false, right)] {
                    let selected = left_selected == is_left;
                    let fill = if selected { MINT } else { Color32::TRANSPARENT };
                    let text = if selected {
                        Color32::from_rgb(28, 83, 96)
                    } else {
                        Color32::from_rgb(58, 87, 95)
                    };
                    let stroke = if selected {
                        Stroke::new(1.0, Color32::from_rgb(205, 225, 229))
                    } else {
                        Stroke::new(1.0, Color32::from_rgb(220, 232, 235))
                    };
                    let button =
                        egui::Button::new(RichText::new(label).size(8.8).strong().color(text))
                            .fill(fill)
                            .stroke(stroke)
                            .corner_radius(CornerRadius::same(7))
                            .min_size(Vec2::new(78.0, 24.0));
                    if ui.add(button).clicked() {
                        on_change(is_left);
                    }
                }
            });
        });
}

fn draw_path_row(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.add_sized(
            Vec2::new(LABEL_WIDTH, CONTROL_HEIGHT),
            egui::Label::new(RichText::new(label).size(8.8)),
        );
        ui.add_sized(
            Vec2::new((ui.available_width() - 46.0).max(140.0), CONTROL_HEIGHT),
            TextEdit::singleline(value)
                .desired_width(f32::INFINITY)
                .hint_text("选择文件"),
        );
        if draw_secondary_button(ui, "选择", 46.0) {
            if let Some(path) = FileDialog::new()
                .pick_file()
                .map(|p| p.display().to_string())
            {
                *value = path;
            }
        }
    });
}

fn draw_secret_row(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.add_sized(
            Vec2::new(LABEL_WIDTH, CONTROL_HEIGHT),
            egui::Label::new(RichText::new(label).size(8.8)),
        );
        ui.add_sized(
            Vec2::new(
                (ui.available_width() - LABEL_WIDTH).max(150.0),
                CONTROL_HEIGHT,
            ),
            TextEdit::singleline(value)
                .password(true)
                .desired_width(f32::INFINITY)
                .hint_text("输入密码"),
        );
    });
}

fn draw_password_field(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.label(RichText::new(label).size(8.8));
    ui.add_sized(
        Vec2::new(ui.available_width(), CONTROL_HEIGHT),
        TextEdit::singleline(value)
            .password(true)
            .desired_width(f32::INFINITY)
            .hint_text("输入密码"),
    );
}

fn draw_primary_button(ui: &mut egui::Ui, label: &str, width: Option<f32>) -> bool {
    ui.add(
        egui::Button::new(
            RichText::new(label)
                .size(9.6)
                .strong()
                .color(Color32::from_rgb(248, 252, 252)),
        )
        .fill(PRIMARY)
        .stroke(Stroke::new(1.0, PRIMARY_HOVER))
        .corner_radius(CornerRadius::same(8))
        .min_size(Vec2::new(width.unwrap_or(72.0), ACTION_HEIGHT)),
    )
    .clicked()
}

fn draw_secondary_button(ui: &mut egui::Ui, label: &str, width: f32) -> bool {
    ui.add(
        egui::Button::new(RichText::new(label).size(8.5).strong().color(PRIMARY))
            .fill(INPUT)
            .stroke(Stroke::new(1.0, BORDER))
            .corner_radius(CornerRadius::same(8))
            .min_size(Vec2::new(width, CONTROL_HEIGHT)),
    )
    .clicked()
}

fn build_crypto_output(directory: &str, input: &str, mode: CryptoMode) -> String {
    let name = file_name(input);
    match mode {
        CryptoMode::Encrypt => join_output(directory, &format!("{name}.enc")),
        CryptoMode::Decrypt => {
            if name.to_lowercase().ends_with(".enc") {
                join_output(directory, &name[..name.len() - 4])
            } else {
                join_output(directory, &format!("{name}.dec"))
            }
        }
    }
}

fn build_stego_output(directory: &str, input: &str, mode: StegoModeUi) -> String {
    let name = file_name(input);
    match mode {
        StegoModeUi::Embed => join_output(directory, &format!("{name}.ste")),
        StegoModeUi::Extract => {
            if name.to_lowercase().ends_with(".ste") {
                join_output(directory, &name[..name.len() - 4])
            } else {
                join_output(directory, &format!("{name}.out"))
            }
        }
    }
}

fn join_output(directory: &str, file_name: &str) -> String {
    PathBuf::from(directory)
        .join(file_name)
        .to_string_lossy()
        .to_string()
}

fn ensure_available_output_path(path: String) -> (String, bool) {
    let original = PathBuf::from(&path);
    if !original.exists() {
        return (path, false);
    }

    let parent = original
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let stem = original
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("output");
    let ext = original.extension().and_then(|value| value.to_str());

    for index in 1..1000 {
        let file_name = match ext {
            Some(ext) if !ext.is_empty() => format!("{stem} ({index}).{ext}"),
            _ => format!("{stem} ({index})"),
        };
        let candidate = parent.join(file_name);
        if !candidate.exists() {
            return (candidate.to_string_lossy().to_string(), true);
        }
    }

    (path, false)
}

fn file_name(path: &str) -> String {
    path.replace('\\', "/")
        .split('/')
        .next_back()
        .unwrap_or_default()
        .to_string()
}

fn validate_crypto_inputs(state: &CryptoState) -> Result<(), &'static str> {
    if state.input_path.trim().is_empty() {
        return Err(if state.mode == CryptoMode::Encrypt {
            "请先选择源文件"
        } else {
            "请先选择加密文件"
        });
    }
    if state.password.trim().is_empty() {
        return Err("请输入主密码");
    }
    if state.id_password.trim().is_empty() {
        return Err("请输入 ID 密码");
    }
    let path = Path::new(state.input_path.trim());
    if !path.exists() || !path.is_file() {
        return Err("所选文件不可用");
    }
    if state.mode == CryptoMode::Decrypt && !has_file_prefix(path, CRYPTO_HEADER) {
        return Err("所选文件不是有效的加密文件");
    }
    Ok(())
}

fn validate_stego_inputs(state: &StegoState) -> Result<(), &'static str> {
    if state.carrier_path.trim().is_empty() {
        return Err("请先选择载体文件");
    }
    if state.mode == StegoModeUi::Embed && state.payload_path.trim().is_empty() {
        return Err("请先选择隐藏文件");
    }
    if state.password.trim().is_empty() {
        return Err("请输入隐写密码");
    }
    let carrier = Path::new(state.carrier_path.trim());
    if !carrier.exists() || !carrier.is_file() {
        return Err("载体文件不可用");
    }
    if state.mode == StegoModeUi::Embed {
        let payload = Path::new(state.payload_path.trim());
        if !payload.exists() || !payload.is_file() {
            return Err("隐藏文件不可用");
        }
    }
    Ok(())
}

fn has_file_prefix(path: &Path, prefix: &[u8]) -> bool {
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return false,
    };
    let mut bytes = vec![0_u8; prefix.len()];
    if file.read_exact(&mut bytes).is_err() {
        return false;
    }
    bytes == prefix
}

fn friendly_error(error: &VaultError) -> String {
    match error {
        VaultError::InvalidInput(message) => {
            let lower = message.to_lowercase();
            if lower.contains("main password is required") {
                "请输入主密码".into()
            } else if lower.contains("id password is required") {
                "请输入 ID 密码".into()
            } else if lower.contains("password") {
                "密码不正确或内容不完整".into()
            } else {
                "输入内容无效，请检查文件和密码".into()
            }
        }
        VaultError::Io(message) => {
            let lower = message.to_lowercase();
            if lower.contains("permission denied") {
                "没有足够的文件访问权限".into()
            } else if lower.contains("no such file")
                || lower.contains("cannot find")
                || lower.contains("not found")
            {
                "未找到所选文件".into()
            } else {
                "文件读写失败，请检查路径和权限".into()
            }
        }
        VaultError::Crypto(_) => "加解密失败，请确认密码和文件内容".into(),
        VaultError::Stego(_) => "隐写处理失败，请确认文件和隐写密码".into(),
        VaultError::Unsupported(_) => "当前操作暂不支持".into(),
    }
}

fn success_message(path: &str, renamed: bool) -> String {
    if renamed {
        format!("已完成：{}（已自动避让重名）", file_name(path))
    } else {
        format!("已完成：{}", file_name(path))
    }
}
