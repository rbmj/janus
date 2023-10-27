use nih_plug_egui::{create_egui_editor, egui, EguiState};
use nih_plug::prelude::*;
use egui::Context;
use egui::widgets;
use std::sync::{Arc, mpsc::SyncSender};
use piano_keyboard::{KeyboardBuilder, Element};
use crate::JanusParams;

fn key_to_notenum(k: egui::Key) -> Option<i8> {
    match k {
        egui::Key::A => Some(janus::midi_const::C4 as i8),
        egui::Key::S => Some(janus::midi_const::D4 as i8),
        egui::Key::D => Some(janus::midi_const::E4 as i8),
        egui::Key::F => Some(janus::midi_const::F4 as i8),
        egui::Key::G => Some(janus::midi_const::G4 as i8),
        egui::Key::H => Some(janus::midi_const::A4 as i8),
        egui::Key::J => Some(janus::midi_const::B4 as i8),
        egui::Key::K => Some(janus::midi_const::C5 as i8),
        egui::Key::L => Some(janus::midi_const::D5 as i8),

        egui::Key::W => Some(janus::midi_const::Db4 as i8),
        egui::Key::E => Some(janus::midi_const::Eb4 as i8),
        egui::Key::T => Some(janus::midi_const::Gb4 as i8),
        egui::Key::Y => Some(janus::midi_const::Ab4 as i8),
        egui::Key::U => Some(janus::midi_const::Bb4 as i8),
        egui::Key::O => Some(janus::midi_const::Db5 as i8),
        egui::Key::P => Some(janus::midi_const::Eb5 as i8),
        _ => None
    }
}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<EguiState> {
    EguiState::from_size(1000, 600)
}

struct JanusEditor {
    params: Arc<JanusParams>,
    channel: SyncSender<i8>,
    last_note: Option<i8>
}

impl JanusEditor {
    pub fn new(p: Arc<JanusParams>, c: SyncSender<i8>) -> Self {
        JanusEditor {
            params: p,
            channel: c,
            last_note: None
        }
    }
    fn param_slider<'a>(setter: &'a ParamSetter, param: &'a IntParam) -> egui::widgets::Slider<'a> {
        let range = param.range();
        let range2 = range; //need a copy to move into the other closure...
        let (min, max) = match range {
            IntRange::Linear{min: x, max: y} => (x, y),
            IntRange::Reversed(IntRange::Linear{min: x, max: y}) => (*x, *y),
            _ => std::unreachable!()
        };
        widgets::Slider::from_get_set(min as f64 ..= max as f64,
            |new_value| {
                match new_value {
                    Some(value) => {
                        setter.begin_set_parameter(param);
                        setter.set_parameter(param, value as i32);
                        setter.end_set_parameter(param);
                        value
                    }
                    None => param.value() as f64
                }
            })
            .integer()
            .show_value(false)
            .suffix(param.unit())
            .custom_parser(move |s| {
                param.string_to_normalized_value(s).map(|x| range.unnormalize(x) as f64)
            })
            .custom_formatter(move |f, _| {
                param.normalized_value_to_string(
                    range2.normalize(f as i32),
                    false)
            })
    }
    pub fn update(&mut self, egui_ctx: &Context, setter: &ParamSetter) {
        egui::TopBottomPanel::bottom("keyboard").show(egui_ctx, |ui| {
            ui.input(|i| {
                for evt in i.events.iter() {
                    if let egui::Event::Key{key, pressed, repeat, ..} = evt {
                        if *repeat { continue; }
                        if let Some(mut k) = key_to_notenum(*key) {
                            if !(*pressed) {
                                k += -128; //Note off
                            }
                            let _ = self.channel.try_send(k);
                        }
                    }
                }
            });
            let keyboard = KeyboardBuilder::new()
                .white_black_gap_present(false)
                .set_width(ui.available_width() as u16)
                .and_then(|x| x.standard_piano(49))
                .map(|x| x.build2d());
            match keyboard {
                Ok(kbd) => {
                    let response = ui.allocate_response(egui::vec2(kbd.width as f32, kbd.height as f32),
                        egui::Sense::click_and_drag());
                    let dragged = response.dragged();
                    let cursor = response.rect.min;
                    for (i, k) in kbd.iter().enumerate() {
                        match k {
                            Element::WhiteKey{wide, small, blind} => {
                                let mut points : Vec<egui::Pos2> = Vec::with_capacity(10);
                                points.push(egui::pos2(
                                    cursor.x + small.x as f32,
                                    cursor.y + small.y as f32));
                                points.push(egui::pos2(
                                    cursor.x + (small.x + small.width) as f32,
                                    cursor.y + small.y as f32));
                                points.push(egui::pos2(
                                    cursor.x + (small.x + small.width) as f32,
                                    cursor.y + (small.y + small.height) as f32));
                                points.push(egui::pos2(
                                    cursor.x + (small.x + small.width) as f32,
                                    cursor.y + (small.y + small.height) as f32));
                                points.push(egui::pos2(
                                    cursor.x + (wide.x + wide.width) as f32,
                                    cursor.y + wide.y as f32));
                                points.push(egui::pos2(
                                    cursor.x + (wide.x + wide.width) as f32,
                                    cursor.y + (wide.y + wide.height) as f32));
                                points.push(egui::pos2(
                                    cursor.x + wide.x as f32,
                                    cursor.y + (wide.y + wide.height) as f32));
                                points.push(egui::pos2(
                                    cursor.x + wide.x as f32,
                                    cursor.y + wide.y as f32));
                                points.push(egui::pos2(
                                    cursor.x + small.x as f32,
                                    cursor.y + (small.y + small.height) as f32));
                                points.dedup();
                                ui.painter().add(egui::epaint::PathShape{
                                    points: points,
                                    closed: true,
                                    fill: egui::epaint::Color32::WHITE,
                                    stroke: egui::epaint::Stroke {
                                        width: 1.0,
                                        color: egui::epaint::Color32::GRAY
                                    }
                                });
                            },
                            Element::BlackKey(r) => {
                                // painting rects doesn't seem to render properly
                                // so we'll use a path instead...
                                // the extra memory allocations hurt my soul but oh well
                                let mut points : Vec<egui::Pos2> = Vec::with_capacity(4);
                                points.push(egui::pos2(
                                    cursor.x + r.x as f32,
                                    cursor.y + r.y as f32));
                                points.push(egui::pos2(
                                    cursor.x + (r.x + r.width) as f32,
                                    cursor.y + r.y as f32));
                                points.push(egui::pos2(
                                    cursor.x + (r.x + r.width) as f32,
                                    cursor.y + (r.y + r.height) as f32));
                                points.push(egui::pos2(
                                    cursor.x + r.x as f32,
                                    cursor.y + (r.y + r.height) as f32));
                                ui.painter().add(egui::epaint::PathShape{
                                    points: points,
                                    closed: true,
                                    fill: egui::epaint::Color32::BLACK,
                                    stroke: egui::epaint::Stroke {
                                        width: 1.0,
                                        color: egui::epaint::Color32::GRAY
                                    }
                                });
                            }
                        }
                    }
                }
                Err(s) => {
                    nih_log!("{}", s);
                }
            }
        });
        egui::CentralPanel::default().show(egui_ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Oscillator 1");
                        ui.horizontal(|ui| {
                            ui.vertical( |ui| {
                                ui.horizontal( |ui| {
                                    ui.label("Shape");
                                    ui.add(Self::param_slider(setter, &self.params.osc1_shape));
                                });
                                //Add widgets for tune, etc.
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.osc1_sin).vertical());
                                ui.label("Sin");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.osc1_tri).vertical());
                                ui.label("Tri");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.osc1_sq).vertical());
                                ui.label("Sq");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.osc1_saw).vertical());
                                ui.label("Saw");
                            });
                        });
                    });
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.label("Filter 1 (SVF)");
                        ui.horizontal(|ui| {
                            ui.vertical( |ui| {
                                ui.horizontal( |ui| {
                                    ui.label("Cutoff");
                                    ui.add(Self::param_slider(setter, &self.params.filt_cutoff));
                                });
                                ui.horizontal( |ui| {
                                    ui.label("Resonance");
                                    ui.add(Self::param_slider(setter, &self.params.filt_res));
                                });
                                ui.horizontal( |ui| {
                                    ui.label("Env Mod");
                                    ui.add(Self::param_slider(setter, &self.params.filt_env));
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Keyboard");
                                    ui.add(Self::param_slider(setter, &self.params.filt_kbd));
                                });
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.filt_low).vertical());
                                ui.label("Low");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.filt_band).vertical());
                                ui.label("Band");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.filt_high).vertical());
                                ui.label("High");
                            });
                        });
                    });
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Envelope 1 (VCF)");
                        ui.horizontal(|ui| {
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.env_vcf_a).vertical());
                                ui.label("Attack");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.env_vcf_d).vertical());
                                ui.label("Decay");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.env_vcf_s).vertical());
                                ui.label("Sustain");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.env_vcf_r).vertical());
                                ui.label("Release");
                            });
                        });
                    });
                    ui.vertical(|ui| {
                        ui.label("Envelope 2 (VCA)");
                        ui.horizontal(|ui| {
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.env_vca_a).vertical());
                                ui.label("Attack");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.env_vca_d).vertical());
                                ui.label("Decay");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.env_vca_s).vertical());
                                ui.label("Sustain");
                            });
                            ui.vertical( |ui| {
                                ui.add(Self::param_slider(setter, &self.params.env_vca_r).vertical());
                                ui.label("Release");
                            });
                        });
                    });
                });
            });
        });
    }
    pub fn update_helper(egui_ctx: &Context, setter: &ParamSetter, state: &mut Self) {
        state.update(egui_ctx, setter)
    }
}

pub(crate) fn create(params: Arc<JanusParams>, tx: SyncSender<i8>) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        params.editor_state.clone(),
        JanusEditor::new(params, tx),
        |_, _| {},
        JanusEditor::update_helper
    )
}
