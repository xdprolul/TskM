use dirs_next::data_dir;
use eframe::egui;
use eframe::{App, Frame, NativeOptions};
use egui::{CentralPanel, Color32, Context, Key, Modifiers, ScrollArea, TextEdit, Vec2};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
struct Task {
    id: usize,
    text: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Section {
    name: String,
    tasks: Vec<Task>,
    new_task_text: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Project {
    name: String,
    sections: Vec<Section>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SectionFocus {
    Header,
    Task(usize),
    NewTaskLine,
    Notes,
}

#[derive(Serialize, Deserialize)]
struct SaveData {
    projects: Vec<Project>,
    next_id: usize,
    notes: String,
}

struct TaskApp {
    projects: Vec<Project>,
    selected_project: usize,
    next_id: usize,
    selected_section: usize,
    focus: SectionFocus,
    notes: String,
    notes_header: String,
}

fn data_file_path() -> PathBuf {
    let base = data_dir().expect("No data_dir available on this plarform");
    let dir = base.join("TskM");
    fs::create_dir_all(&dir).ok();
    dir.join("TskM.json")
}

impl Default for TaskApp {
    fn default() -> Self {
        if let Some(app) = TaskApp::load_from_file() {
            return app;
        }

        let mut app = Self {
            projects: vec![Project {
                name: "project-1".to_string(),
                sections: vec![
                    Section {
                        name: "pending-".to_string(),
                        tasks: vec![
                            Task {
                                id: 1,
                                text: "private lobby?".into(),
                            },
                            Task {
                                id: 2,
                                text: "invite email".into(),
                            },
                            Task {
                                id: 3,
                                text: "seat change option".into(),
                            },
                        ],
                        new_task_text: String::new(),
                    },
                    Section {
                        name: "improvements to do-".to_string(),
                        tasks: vec![
                            Task {
                                id: 4,
                                text: "control panel orientation on hand after loose app focus"
                                    .into(),
                            },
                            Task {
                                id: 5,
                                text: "hand gestures".into(),
                            },
                        ],
                        new_task_text: String::new(),
                    },
                    Section {
                        name: "bugs-".to_string(),
                        tasks: vec![Task {
                            id: 6,
                            text: "control panel on controller changes orientation".into(),
                        }],
                        new_task_text: String::new(),
                    },
                    Section {
                        name: "pushed updates for testing-".to_string(),
                        tasks: vec![
                            Task {
                                id: 7,
                                text: "fullscreen annotation for sphere".into(),
                            },
                            Task {
                                id: 8,
                                text: "annotation panel position for all theatres".into(),
                            },
                            Task {
                                id: 9,
                                text: "sphere screen integration".into(),
                            },
                            Task {
                                id: 10,
                                text: "change theatre".into(),
                            },
                        ],
                        new_task_text: String::new(),
                    },
                ],
            }],
            selected_project: 0,
            next_id: 11,
            selected_section: 0,
            focus: SectionFocus::Header,
            notes: String::new(),
            notes_header: "notes-".to_string(),
        };

        if app.projects.is_empty() {
            app.selected_project = 0;
            app.selected_section = 0;
            app.focus = SectionFocus::NewTaskLine;
            return app;
        }

        if !app.projects[0].sections.is_empty() && !app.projects[0].sections[0].tasks.is_empty() {
            app.focus = SectionFocus::Task(0);
        } else {
            app.focus = SectionFocus::NewTaskLine;
        }

        app
    }
}

impl TaskApp {
    fn move_selected_to_section(&mut self, target_section: usize) {
        let project = &mut self.projects[self.selected_project];
        if target_section >= project.sections.len() {
            return;
        }
        let from_sec = self.selected_section;
        let idx = match self.focus {
            SectionFocus::Task(i) => i,
            _ => return,
        };

        if from_sec == target_section {
            return;
        }

        if idx
            >= self.projects[self.selected_project].sections[from_sec]
                .tasks
                .len()
        {
            return;
        }

        let task = self.projects[self.selected_project].sections[from_sec]
            .tasks
            .remove(idx);

        let new_idx = self.projects[self.selected_project].sections[target_section]
            .tasks
            .len();
        self.projects[self.selected_project].sections[target_section]
            .tasks
            .insert(new_idx, task);

        self.selected_section = target_section;
        self.focus = SectionFocus::Task(new_idx);
    }

    fn delete_selected_task(&mut self) {
        let sec_idx = self.selected_section;
        if sec_idx >= self.projects[self.selected_project].sections.len() {
            return;
        }

        let idx = match self.focus {
            SectionFocus::Task(i) => i,
            _ => return,
        };

        let project = &mut self.projects[self.selected_project];
        if idx >= project.sections[sec_idx].tasks.len() {
            return;
        }

        project.sections[sec_idx].tasks.remove(idx);

        if project.sections[sec_idx].tasks.is_empty() {
            self.focus = SectionFocus::NewTaskLine;
        } else if idx < project.sections[sec_idx].tasks.len() {
            self.focus = SectionFocus::Task(idx);
        } else {
            self.focus = SectionFocus::Task(project.sections[sec_idx].tasks.len() - 1);
        }
    }

    fn save_to_file(&self) {
        let path = data_file_path();
        let data = SaveData {
            projects: self.projects.clone(),
            next_id: self.next_id,
            notes: self.notes.clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&data) {
            let _ = fs::write(path, json);
        }
    }

    fn load_from_file() -> Option<Self> {
        let path = data_file_path();
        let text = fs::read_to_string(path).ok()?;
        let data: SaveData = serde_json::from_str(&text).ok()?;

        let mut app = Self {
            projects: data.projects,
            selected_project: 0,
            next_id: data.next_id,
            selected_section: 0,
            focus: SectionFocus::Header,
            notes: data.notes,
            notes_header: "notes-".to_string(),
        };

        if app.projects.is_empty() {
            app.focus = SectionFocus::NewTaskLine;
            return Some(app);
        }

        if !app.projects[0].sections.is_empty() && !app.projects[0].sections[0].tasks.is_empty() {
            app.focus = SectionFocus::Task(0);
        } else {
            app.focus = SectionFocus::NewTaskLine;
        }

        Some(app)
    }
}

impl App for TaskApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
				ctx.set_visuals(egui::Visuals::dark());

				if self.projects.is_empty() {
    self.show_welcome = true;

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.3);

            ui.heading("TskM");

            ui.add_space(20.0);

            if ui.button("Create Project").clicked() {
                self.projects.push(Project {
                    name: "Project-1".to_string(),
                    sections: vec![
                        Section {
                            name: "pending-".to_string(),
                            tasks: vec![],
                            new_task_text: String::new(),
                        },
                        Section {
                            name: "bugs-".to_string(),
                            tasks: vec![],
                            new_task_text: String::new(),
                        },
                    ],
                });

                self.selected_project = 0;
                self.selected_section = 0;
                self.focus = SectionFocus::NewTaskLine;
                self.show_welcome = false;
            }
        });
    });

    return;
}
				
	
        let should_close = ctx.input(|i| i.viewport().close_requested());
        if should_close {
            self.save_to_file();
        }

        let save_pressed =
            ctx.input(|i| i.key_pressed(Key::S) && i.modifiers.matches_logically(Modifiers::CTRL));
        if save_pressed {
            self.save_to_file();
        }

        let input = ctx.input(|i| i.clone());
       	if self.selected_project >= self.projects.len() {
					self.selected_project = 0;
				}

        let project_ref = &self.projects[self.selected_project];
        let sec_count = project_ref.sections.len();
        //if sec_count == 0 {
        //    return;
        //}

        let current_sec = self.selected_section.min(sec_count - 1);
        self.selected_section = current_sec;
        let current_tasks_len = project_ref.sections[current_sec].tasks.len();

        if input.modifiers.ctrl {
            if input.key_pressed(Key::ArrowUp) {
                let target = if self.selected_section == 0 {
                    sec_count - 1
                } else {
                    self.selected_section - 1
                };
                self.move_selected_to_section(target);
                ctx.memory_mut(|m| m.request_focus(egui::Id::NULL));
            }

            if input.key_pressed(Key::ArrowDown) {
                let target = if self.selected_section + 1 >= sec_count {
                    0
                } else {
                    self.selected_section + 1
                };
                self.move_selected_to_section(target);
                ctx.memory_mut(|m| m.request_focus(egui::Id::NULL));
            }

            if input.key_pressed(Key::ArrowLeft) {
                self.focus = SectionFocus::Notes;
            }

            if input.key_pressed(Key::ArrowRight) {
                self.focus = SectionFocus::NewTaskLine;
            }
        }

        if !input.modifiers.ctrl {
            if input.key_pressed(Key::ArrowUp) {
                match self.focus {
                    SectionFocus::Task(idx) => {
                        if idx > 0 {
                            self.focus = SectionFocus::Task(idx - 1);
                        } else if current_sec > 0 {
                            self.selected_section -= 1;
                            self.focus = SectionFocus::NewTaskLine;
                        } else {
                            self.selected_section = sec_count - 1;
                            self.focus = SectionFocus::NewTaskLine;
                        }
                        ctx.memory_mut(|m| m.request_focus(egui::Id::NULL));
                    }
                    SectionFocus::NewTaskLine => {
                        if current_tasks_len > 0 {
                            self.focus = SectionFocus::Task(current_tasks_len - 1);
                        } else if current_sec > 0 {
                            self.selected_section -= 1;
                            self.focus = SectionFocus::NewTaskLine;
                        } else {
                            self.selected_section = sec_count - 1;
                            self.focus = SectionFocus::NewTaskLine;
                        }
                        ctx.memory_mut(|m| m.request_focus(egui::Id::NULL));
                    }
                    SectionFocus::Header => {}
                    SectionFocus::Notes => {}
                }
            }
            if input.key_pressed(Key::ArrowDown) {
                match self.focus {
                    SectionFocus::Task(idx) => {
                        if idx + 1 < current_tasks_len {
                            self.focus = SectionFocus::Task(idx + 1);
                        } else {
                            self.focus = SectionFocus::NewTaskLine;
                        }
                    }
                    SectionFocus::NewTaskLine => {
                        if current_sec + 1 < sec_count {
                            self.selected_section += 1;
                            let next_project = &self.projects[self.selected_project];
                            let next_tasks_len =
                                next_project.sections[self.selected_section].tasks.len();
                            self.focus = if next_tasks_len > 0 {
                                SectionFocus::Task(0)
                            } else {
                                SectionFocus::NewTaskLine
                            };
                        } else {
                            self.selected_section = 0;
                            let first_project = &self.projects[self.selected_project];
                            let first_tasks_len = first_project.sections[0].tasks.len();
                            self.focus = if first_tasks_len > 0 {
                                SectionFocus::Task(0)
                            } else {
                                SectionFocus::NewTaskLine
                            };
                        }
                        ctx.memory_mut(|m| m.request_focus(egui::Id::NULL));
                    }
                    SectionFocus::Header => {}
                    SectionFocus::Notes => {}
                }
            }
            if input.key_pressed(Key::Delete) {
                self.delete_selected_task();
            }
            if input.key_pressed(Key::F2) {
                self.focus = SectionFocus::Header;
            }
        }

        egui::SidePanel::left("notes_panel")
            .resizable(true)
            .default_width(450.0)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(&self.notes_header)
                        .color(egui::Color32::LIGHT_BLUE)
                        .strong(),
                );

                ui.add_space(6.0);

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add(
                            TextEdit::multiline(&mut self.notes)
                                .frame(false)
                                .margin(Vec2::ZERO)
                                .desired_width(ui.available_width()),
                        );
                    });
            });

        CentralPanel::default().show(ctx, |ui| {
            // tab row
                ui.horizontal(|ui| {
                    let mut to_delete = None;

                    for (i, project) in self.projects.iter().enumerate() {
                        let is_selected = i == self.selected_project;

                        let text = format!("{}   x", project.name);
                        let resp = ui.add(egui::Button::new(text).frame(false));
                        if is_selected {
                            ui.painter().rect_filled(
                                resp.rect.expand(2.0),
                                3.0,
                                egui::Color32::TRANSPARENT,
                            );
                        }

                    let x_w = 18.0;
                    let x_rect = egui::Rect::from_min_max(
                        egui::pos2(resp.rect.max.x - x_w, resp.rect.min.y),
                        resp.rect.max,
                    );

                    let pointer_pos = ui.input(|inp| inp.pointer.hover_pos());
                    let x_hovered = pointer_pos.map_or(false, |p| x_rect.contains(p));

                    // if resp.hovered() && x_hovered {
                    //     ui.painter().rect_filled(
                    //         x_rect.shrink(2.0),
                    //         2.0,
                    //         egui::Color32::from_rgb(70,70,70),
                    //     );
                    // }

                    if resp.clicked() {
                        if x_hovered {
                            to_delete = Some(i);
                        } else {
                            self.selected_project = i;
                            self.selected_section = 0;

                            let first_tasks_len = self.projects[i].sections[0].tasks.len();
                            self.focus = if first_tasks_len > 0 {
                                SectionFocus::Task(0)
                            } else {
                                SectionFocus::NewTaskLine
                            };
                        }
                    }
                }

                let mut deleted_last_project = false;

                if let Some(i) = to_delete {
                    self.projects.remove(i);

                    if self.projects.is_empty() {
                        deleted_last_project = true;
                    } else {
                        if self.selected_project >= self.projects.len() {
                            self.selected_project = self.projects.len() - 1;
                            self.selected_section = 0;
                            self.focus = SectionFocus::NewTaskLine;
                        }
                    }
                }

                if deleted_last_project {
                    ui.separator();
                    return;
                }

                if ui.button("+").clicked() {
                    let new_index = self.projects.len() + 1;
                    self.projects.push(Project {
                        name: format!("project-{}", new_index),
                        sections: vec![
                            Section {
                                name: "pending-".to_string(),
                                tasks: vec![],
                                new_task_text: String::new(),
                            },
                            Section {
                                name: "improvements to do-".to_string(),
                                tasks: vec![],
                                new_task_text: String::new(),
                            },
                            Section {
                                name: "bugs-".to_string(),
                                tasks: vec![],
                                new_task_text: String::new(),
                            },
                            Section {
                                name: "pushed updates-".to_string(),
                                tasks: vec![],
                                new_task_text: String::new(),
                            },
                        ],
                    });
                    self.selected_project = self.projects.len() - 1;
                    self.selected_section = 0;
                    self.focus = SectionFocus::NewTaskLine;
                }
            });

            ui.separator();

            if self.projects.is_empty() {
                return;
            }
            let project = &mut self.projects[self.selected_project];

            ScrollArea::vertical().show(ui, |ui| {
                for (section_idx, section) in project.sections.iter_mut().enumerate() {
                    ui.add_space(10.0);
                    let is_current_section = section_idx == self.selected_section;
                    let header_id = ui.id().with(("header", section_idx));
                    ui.horizontal(|ui| {
                        let header_selected =
                            is_current_section && matches!(self.focus, SectionFocus::Header);

                        let frame = egui::Frame::NONE.fill(if header_selected {
                            Color32::from_rgb(60, 80, 120)
                        } else {
                            Color32::TRANSPARENT
                        });

                        frame.show(ui, |ui: &mut egui::Ui| {
                            if header_selected {
                                // ui.memory_mut(|m| m.request_focus(header_id));
                                let _resp = ui.add(
                                    TextEdit::singleline(&mut section.name)
                                        .id(header_id)
                                        .frame(false)
                                        .desired_width(ui.available_width()),
                                );
                                if ui.input(|i: &egui::InputState| i.key_pressed(Key::Enter)) {
                                    self.focus = if !section.tasks.is_empty() {
                                        SectionFocus::Task(0)
                                    } else {
                                        SectionFocus::NewTaskLine
                                    }
                                }
                            } else {
                                ui.label(
                                    egui::RichText::new(&section.name)
                                        .color(egui::Color32::LIGHT_BLUE)
                                        .strong(),
                                );
                            }
                        });
                    });

                    for (idx, task) in section.tasks.iter().enumerate() {
                        let is_selected = is_current_section
                            && matches!(self.focus, SectionFocus::Task(i) if i == idx);

                        let frame = egui::Frame::NONE.fill(if is_selected {
                            Color32::from_rgb(60, 80, 120)
                        } else {
                            Color32::TRANSPARENT
                        });

                        frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("::");
                                ui.label(&task.text);
                            });
                        });
                    }

                    let new_task_id = ui.id().with(("new_task", section_idx));

                    ui.horizontal(|ui| {
                        let is_new_task_selected =
                            is_current_section && matches!(self.focus, SectionFocus::NewTaskLine);

                            let frame = egui::Frame::NONE;
                                //.fill (Color32::from_rgb(30,30,30));

                        frame.show(ui, |ui| {
                            let edit = TextEdit::singleline(&mut section.new_task_text)
                                .frame(false)
                                .margin(Vec2::ZERO);

                            // if is_new_task_selected {
                            //     ui.memory_mut(|m| m.request_focus(new_task_id));
                            // }

                            let resp = ui.add(edit.id(new_task_id));

                            if resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                                if !section.new_task_text.trim().is_empty() {
                                    let id = self.next_id;
                                    self.next_id += 1;

                                    section.tasks.push(Task {
                                        id,
                                        text: section.new_task_text.trim().to_string(),
                                    });
                                    section.new_task_text.clear();

                                    if is_current_section {
                                        self.focus = SectionFocus::Task(section.tasks.len() - 1);
                                    }
                                }
                            }
                        });
                    });
                }
            });

                ui.add_space(10.0);
                ui.separator();
            });
    }
}

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
				renderer: eframe::Renderer::Glow,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "TskM",
        options,
        Box::new(|_cc| Ok(Box::new(TaskApp::default()))),
    )
}
