use eframe::egui;
use egui::{Color32, CornerRadius, Pos2, Stroke, Vec2};
use pyo3::prelude::*;
use pyo3::types::PyModule;

const COLOR_OLIVE: Color32 = Color32::from_rgb(122, 132, 76);
const COLOR_PANEL: Color32 = Color32::from_rgb(150, 150, 150);
const COLOR_SCREEN: Color32 = Color32::BLACK;
const COLOR_GRILLE: Color32 = Color32::from_gray(35);
const COLOR_GREEN: Color32 = Color32::from_rgb(70, 220, 40);
const COLOR_RED: Color32 = Color32::from_rgb(220, 40, 40);
const COLOR_ETIQUETA: Color32 = Color32::from_rgb(235, 255, 225);

const DIRECCIONES: [(&str, f32, &str); 8] = [
    ("w", 0.0, "F"),
    ("e", 45.0, "e"),
    ("d", 90.0, "R"),
    ("c", 135.0, "c"),
    ("s", 180.0, "B"),
    ("z", 225.0, "z"),
    ("a", 270.0, "L"),
    ("q", 315.0, "q"),
];

struct PyController {
    module: Py<PyModule>,
}

impl PyController {
    fn new() -> PyResult<Self> {
        Python::attach(|py| {
            let sys = py.import("sys")?;
            sys.getattr("path")?.call_method1("insert", (0, "."))?;

            let module = PyModule::import(py, "controller")?;
            Ok(PyController {
                module: module.unbind(),
            })
        })
    }

    fn conectar(&self, puerto: &str) -> (bool, String) {
        Python::attach(|py| {
            self.module
                .bind(py)
                .call_method1("conectar", (puerto,))
                .and_then(|r| r.extract::<(bool, String)>())
                .unwrap_or_else(|e| (false, format!("Error de Python: {e}")))
        })
    }

    fn enviar(&self, comando: &str) -> String {
        Python::attach(|py| {
            self.module
                .bind(py)
                .call_method1("enviar", (comando,))
                .and_then(|r| r.extract::<String>())
                .unwrap_or_else(|e| format!("Error de Python: {e}"))
        })
    }

    fn cambiar_velocidad(&self, tecla: &str) -> String {
        Python::attach(|py| {
            self.module
                .bind(py)
                .call_method1("cambiar_velocidad", (tecla,))
                .and_then(|r| r.extract::<String>())
                .unwrap_or_else(|e| format!("Error de Python: {e}"))
        })
    }

    fn turbo(&self, activar: bool) -> String {
        Python::attach(|py| {
            self.module
                .bind(py)
                .call_method1("turbo", (activar,))
                .and_then(|r| r.extract::<String>())
                .unwrap_or_else(|e| format!("Error de Python: {e}"))
        })
    }

    fn desconectar(&self) -> String {
        Python::attach(|py| {
            self.module
                .bind(py)
                .call_method0("desconectar")
                .and_then(|r| r.extract::<String>())
                .unwrap_or_else(|e| format!("Error de Python: {e}"))
        })
    }
}

struct RobotApp {
    controller: Option<PyController>,
    error_inicial: Option<String>,

    puerto: String,
    conectado: bool,
    nivel_velocidad: i32,
    turbo_activo: bool,
    control_por_teclado: bool,
    mostrar_ajustes: bool,
    ultimo_comando: Option<String>,
    log: Vec<String>,
}

impl Default for RobotApp {
    fn default() -> Self {
        let (controller, error_inicial) = match PyController::new() {
            Ok(c) => (Some(c), None),
            Err(e) => (
                None,
                Some(format!("No se pudo inicializar Python/controller.py: {e}")),
            ),
        };

        Self {
            controller,
            error_inicial,
            puerto: "COM3".to_string(),
            conectado: false,
            nivel_velocidad: 5,
            turbo_activo: false,
            control_por_teclado: false,
            mostrar_ajustes: false,
            ultimo_comando: None,
            log: Vec::new(),
        }
    }
}

impl RobotApp {
    fn registrar(&mut self, mensaje: String) {
        if !mensaje.is_empty() {
            self.log.push(mensaje);
            if self.log.len() > 100 {
                self.log.remove(0);
            }
        }
    }

    fn enviar_comando(&mut self, comando: &str) {
        if let Some(controller) = &self.controller {
            let mensaje = controller.enviar(comando);
            self.registrar(mensaje);
        }
        self.ultimo_comando = Some(comando.to_string());
    }

    fn cambiar_paso_velocidad(&mut self, delta: i32) {
        let nuevo = (self.nivel_velocidad + delta).clamp(0, 9);
        if nuevo != self.nivel_velocidad {
            self.nivel_velocidad = nuevo;
            if let Some(controller) = &self.controller {
                let mensaje = controller.cambiar_velocidad(&nuevo.to_string());
                self.registrar(mensaje);
            }
        }
    }

    fn dibujar_pantalla(&mut self, ui: &mut egui::Ui) {
        // --- Fila de estados ---
        ui.horizontal(|ui| {
            let (circ_rect, _) =
                ui.allocate_exact_size(egui::vec2(34.0, 34.0), egui::Sense::hover());
            ui.painter().circle_filled(
                circ_rect.center(),
                14.0,
                if self.conectado {
                    COLOR_GREEN
                } else {
                    COLOR_RED
                },
            );

            ui.add_space(10.0);
            columna_estado(
                ui,
                "Status",
                if self.conectado {
                    "Conectado"
                } else {
                    "Disconected"
                },
                if self.conectado {
                    COLOR_GREEN
                } else {
                    COLOR_RED
                },
            );
            ui.add_space(46.0);

            let en_movimiento = matches!(self.ultimo_comando.as_deref(), Some(c) if c != "S");
            columna_estado(
                ui,
                "Rover Status",
                if en_movimiento { "Moving" } else { "Still" },
                COLOR_GREEN,
            );
            ui.add_space(46.0);

            columna_estado(
                ui,
                "PC Bluetooth",
                if self.conectado { "On" } else { "Off" },
                if self.conectado {
                    COLOR_GREEN
                } else {
                    COLOR_RED
                },
            );
        });

        ui.add_space(36.0);

        ui.vertical_centered(|ui| {
            let tamano = 500.;
            panel_direccional(ui, tamano, self);
        });

        ui.add_space(14.0);

        ui.horizontal(|ui| {
            ui.colored_label(COLOR_GREEN, format!("Speed:   {}", self.nivel_velocidad));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                dibujar_alerta_turbo(ui, self.turbo_activo);
            });
        });
    }
}

fn columna_estado(ui: &mut egui::Ui, titulo: &str, valor: &str, color: Color32) {
    ui.vertical(|ui| {
        ui.set_width(150.0);
        ui.add(
            egui::Label::new(egui::RichText::new(titulo).color(COLOR_GREEN))
                .wrap_mode(egui::TextWrapMode::Extend),
        );
        ui.add(
            egui::Label::new(
                egui::RichText::new("-".repeat(16))
                    .color(COLOR_GREEN)
                    .monospace(),
            )
            .wrap_mode(egui::TextWrapMode::Extend),
        );
        ui.add(
            egui::Label::new(egui::RichText::new(valor).color(color))
                .wrap_mode(egui::TextWrapMode::Extend),
        );
    });
}

fn dibujar_alerta_turbo(ui: &mut egui::Ui, activo: bool) {
    let color = if activo {
        COLOR_RED
    } else {
        Color32::from_gray(70)
    };
    let (rect, _) = ui.allocate_exact_size(egui::vec2(56.0, 56.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    let c = rect.center();
    let pts = vec![
        Pos2::new(c.x, c.y - 20.0),
        Pos2::new(c.x - 20.0, c.y + 14.0),
        Pos2::new(c.x + 20.0, c.y + 14.0),
    ];
    let mut cerrado = pts.clone();
    cerrado.push(pts[0]);
    painter.add(egui::Shape::line(cerrado, Stroke::new(2.5, color)));
    painter.text(
        egui::pos2(c.x, c.y),
        egui::Align2::CENTER_CENTER,
        "!",
        egui::FontId::monospace(15.0),
        color,
    );
    painter.text(
        egui::pos2(c.x, c.y + 28.0),
        egui::Align2::CENTER_CENTER,
        "T",
        egui::FontId::monospace(14.0),
        color,
    );
}

fn boton_icono(ui: &mut egui::Ui, icono: &str, texto: &str) -> egui::Response {
    ui.vertical(|ui| {
        ui.set_width(80.0);
        let boton = egui::Button::new(egui::RichText::new(icono).size(20.0))
            .min_size(egui::vec2(72.0, 44.0));
        let respuesta = ui.add(boton);
        ui.label(
            egui::RichText::new(texto)
                .size(10.0)
                .color(egui::Color32::BLACK),
        );
        respuesta
    })
    .inner
}

/// Dibuja un dardo/flecha simple apuntando hacia afuera desde `center`, rotado
/// según `angle_deg` (0° = arriba). `r_base` deja un hueco entre el centro y
/// la base de la flecha para que las 8 direcciones no se fusionen en una
/// "flor"/rueda dentada, y `medio_ancho` controla el espacio entre flechas
/// contiguas.
fn draw_dir_arrow(
    painter: &egui::Painter,
    center: Pos2,
    angle_deg: f32,
    color: Color32,
    r_tip: f32,
    r_base: f32,
    medio_ancho: f32,
) {
    let angle = angle_deg.to_radians();
    let (sin, cos) = angle.sin_cos();
    let rotate = |p: Vec2| Vec2::new(p.x * cos - p.y * sin, p.x * sin + p.y * cos);

    let pts = [
        Vec2::new(0.0, -r_tip),
        Vec2::new(medio_ancho, -r_base),
        Vec2::new(-medio_ancho, -r_base),
    ];
    let poligono: Vec<Pos2> = pts.iter().map(|p| center + rotate(*p)).collect();
    painter.add(egui::Shape::convex_polygon(poligono, color, Stroke::NONE));
}

fn panel_direccional(ui: &mut egui::Ui, tamano: f32, estado: &mut RobotApp) {
    //let tamano = 560.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(tamano, tamano), egui::Sense::hover());
    let center = rect.center();
    let painter = ui.painter_at(rect.expand(50.0));

    let r_tip = 138.0;
    let r_base = 60.0;
    let medio_ancho = 18.0;
    let r_centro_click = (r_tip + r_base) / 2.0;
    let r_texto = r_tip + 34.0;

    let mut comando_activo: Option<&str> = None;

    for (etiqueta, angulo, comando) in DIRECCIONES {
        let rad = angulo.to_radians();
        let dir = egui::vec2(rad.sin(), -rad.cos());
        let centro_hit = center + dir * r_centro_click;

        let hit_rect = egui::Rect::from_center_size(centro_hit, egui::vec2(66.0, 66.0));
        let id = ui.id().with(etiqueta);
        let respuesta = ui.interact(hit_rect, id, egui::Sense::click());
        let activo = respuesta.is_pointer_button_down_on();
        if activo {
            comando_activo = Some(comando);
        }

        let color = if activo {
            Color32::from_rgb(160, 255, 110)
        } else {
            COLOR_GREEN
        };
        draw_dir_arrow(&painter, center, angulo, color, r_tip, r_base, medio_ancho);

        let pos_texto = center + dir * r_texto;
        painter.text(
            pos_texto,
            egui::Align2::CENTER_CENTER,
            etiqueta,
            egui::FontId::monospace(18.0),
            COLOR_ETIQUETA,
        );
    }

    match comando_activo {
        Some(cmd) => estado.enviar_comando(cmd),
        None => estado.enviar_comando("S"),
    }

    ui.ctx().request_repaint();
}
impl eframe::App for RobotApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Frame::new()
            .fill(COLOR_OLIVE)
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                if let Some(err) = self.error_inicial.clone() {
                    ui.colored_label(Color32::WHITE, err);
                    return;
                }

                ui.horizontal(|ui| {
                    egui::Frame::new()
                        .fill(COLOR_PANEL)
                        .corner_radius(CornerRadius::same(14))
                        .inner_margin(egui::Margin::same(12))
                        .show(ui, |ui| {
                            // ui.set_width(720.0);
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    if boton_icono(ui, "⚙", "Settings").clicked() {
                                        self.mostrar_ajustes = !self.mostrar_ajustes;
                                    }
                                    if boton_icono(ui, "💡", "Front Light (K)").clicked() {
                                        self.enviar_comando("k");
                                    }
                                    if boton_icono(ui, "🔆", "Rear Light (L)").clicked() {
                                        self.enviar_comando("l");
                                    }
                                    if boton_icono(ui, "📢", "Horn (H)").clicked() {
                                        self.enviar_comando("h");
                                    }
                                    if boton_icono(ui, "+", "Increase Speed").clicked() {
                                        self.cambiar_paso_velocidad(1);
                                    }
                                    if boton_icono(ui, "-", "decrease Speed").clicked() {
                                        self.cambiar_paso_velocidad(-1);
                                    }
                                });

                                ui.add_space(10.0);

                                egui::Frame::new()
                                    .fill(COLOR_SCREEN)
                                    .corner_radius(CornerRadius::same(6))
                                    .inner_margin(egui::Margin::same(14))
                                    .show(ui, |ui| {
                                        //ui.set_min_height(560.0);
                                        self.dibujar_pantalla(ui);
                                    });
                            });
                        });
                });
            });

        if self.mostrar_ajustes {
            let ctx = ui.ctx().clone();
            let mut abierto = self.mostrar_ajustes;
            egui::Window::new("Ajustes de conexión")
                //.collapsible(false)
                //.resizable(false)
                .open(&mut abierto)
                .show(&ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Puerto:");
                        ui.add_enabled(
                            !self.conectado,
                            egui::TextEdit::singleline(&mut self.puerto).desired_width(140.0),
                        );
                    });

                    ui.horizontal(|ui| {
                        if !self.conectado {
                            if ui.button("Conectar").clicked() {
                                if let Some(controller) = &self.controller {
                                    let (ok, mensaje) = controller.conectar(&self.puerto);
                                    self.conectado = ok;
                                    self.registrar(mensaje);
                                }
                            }
                        } else if ui.button("Desconectar").clicked() {
                            if let Some(controller) = &self.controller {
                                let mensaje = controller.desconectar();
                                self.registrar(mensaje);
                            }
                            self.conectado = false;
                        }
                    });

                    ui.separator();

                    if ui.checkbox(&mut self.turbo_activo, "Modo TURBO").changed() {
                        if let Some(controller) = &self.controller {
                            let mensaje = controller.turbo(self.turbo_activo);
                            self.registrar(mensaje);
                        }
                    }

                    ui.checkbox(
                        &mut self.control_por_teclado,
                        "Control con flechas del teclado",
                    );

                    ui.separator();
                    ui.label("Registro:");
                    egui::ScrollArea::vertical()
                        //.max_height(160.0)
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            for linea in &self.log {
                                ui.label(linea);
                            }
                        });
                });
            self.mostrar_ajustes = abierto;
        }

        if self.conectado && self.control_por_teclado {
            let (arriba, abajo, izquierda, derecha) = ui.ctx().input(|i| {
                (
                    i.key_down(egui::Key::ArrowUp),
                    i.key_down(egui::Key::ArrowDown),
                    i.key_down(egui::Key::ArrowLeft),
                    i.key_down(egui::Key::ArrowRight),
                )
            });

            if arriba {
                self.enviar_comando("F");
            } else if abajo {
                self.enviar_comando("B");
            } else if izquierda {
                self.enviar_comando("L");
            } else if derecha {
                self.enviar_comando("R");
            }

            ui.ctx().request_repaint();
        }
    }

    fn on_exit(&mut self) {
        if self.conectado {
            if let Some(controller) = &self.controller {
                controller.desconectar();
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let opciones = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native(
        "Control de Robot Bluetooth",
        opciones,
        Box::new(|_cc| Ok(Box::new(RobotApp::default()))),
    )
}
