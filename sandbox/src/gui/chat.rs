use std::collections::VecDeque;
use std::time::Instant;
use innovus::gfx::color::Color;
use innovus::gfx::Gfx;
use innovus::gfx::mesh::MeshRenderer;
use innovus::tools::Vector;
use crate::gui::render::GuiVertex;
use crate::gui::render::text::{TextBackground, TextLine};
use crate::script::ScriptingEngine;
use crate::tools::asset::AssetPool;
use crate::tools::input::InputState;
use crate::world::World;

pub struct ChatBox {
    anchor: Vector<f32, 2>,
    offset: Vector<f32, 2>,
    message_history: VecDeque<ChatMessage>,
    max_history: usize,
    current_message: TextLine,
    is_open: bool,
    line_spacing: f32,
    background_opacity: f32,
    text_layer: MeshRenderer<GuiVertex>,
}

impl ChatBox {
    pub fn create(gfx: &Gfx, max_history: usize, line_spacing: f32, background_opacity: f32) -> Result<Self, String> {
        Ok(Self {
            anchor: Vector([0.0, 1.0]),
            offset: Vector([4.0, -36.0]),
            message_history: VecDeque::with_capacity(max_history),
            max_history,
            current_message: TextLine::new(
                Vector([0.0, 1.0]),
                Color::White.into(),
                TextBackground::Rectangle {
                    color: Color::Black.with_alpha(background_opacity),
                    margin: Vector([1.0, 0.0]),
                },
                String::new(),
            ),
            is_open: false,
            line_spacing,
            background_opacity,
            text_layer: MeshRenderer::create(gfx),
        })
    }

    pub fn anchor(&self) -> Vector<f32, 2> {
        self.anchor
    }

    pub fn set_anchor(&mut self, anchor: Vector<f32, 2>) {
        self.anchor = anchor;
    }

    pub fn offset(&self) -> Vector<f32, 2> {
        self.offset
    }

    pub fn set_offset(&mut self, offset: Vector<f32, 2>) {
        self.offset = offset;
        self.invalidate();
    }

    pub fn max_history(&self) -> usize {
        self.max_history
    }

    pub fn set_max_history(&mut self, max_history: usize) {
        self.max_history = max_history;
        while self.message_history.len() > max_history {
            self.message_history.pop_front();
        }
        self.message_history.shrink_to(max_history);
        self.invalidate();
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn set_open(&mut self, open: bool) {
        self.is_open = open;
        self.invalidate();
    }

    pub fn line_spacing(&self) -> f32 {
        self.line_spacing
    }

    pub fn set_line_spacing(&mut self, line_spacing: f32) {
        self.line_spacing = line_spacing;
        self.invalidate();
    }

    pub fn background_opacity(&self) -> f32 {
        self.background_opacity
    }

    pub fn set_background_opacity(&mut self, opacity: f32) {
        self.background_opacity = opacity;
        let background = TextBackground::Rectangle {
            color: Color::Black.with_alpha(self.background_opacity),
            margin: Vector([1.0, 0.0]),
        };

        for message in &mut self.message_history {
            message.line_mut().set_background(background.clone());
        }
        self.current_message.set_background(background);
        self.invalidate();
    }

    pub fn invalidate(&mut self) {
        self.text_layer.clear();
    }

    pub fn reload_assets(&mut self) {
        self.current_message.invalidate();
        for message in &mut self.message_history {
            message.line_mut().invalidate();
        }
        self.invalidate();
    }

    pub fn handle_keyboard(&mut self, inputs: &InputState, scripting: &ScriptingEngine, world: &mut World, assets: &AssetPool) -> bool {
        if self.is_open {
            if inputs.key_was_repeated(Key::Backspace) {
                self.current_message.text_mut().pop();
            }
            if !inputs.entered_text().is_empty() {
                self.current_message.text_mut().push_str(inputs.entered_text());
            }
            if inputs.key_was_pressed(Key::Escape) {
                std::mem::take(self.current_message.text_mut());
                self.set_open(false);
            }
            else if inputs.key_was_pressed(Key::Enter) {
                let text = std::mem::take(self.current_message.text_mut());
                if !text.is_empty() {
                    if text.starts_with('/') {
                        match scripting.dispatch_command(&text, world, assets) {
                            Ok(text) => {
                                self.add_plain_message(text, Color::Green);
                            }
                            Err(text) => {
                                self.add_plain_message(text, Color::Red);
                            }
                        }
                    }
                    else {
                        self.add_plain_message(format!("{}: {text}", world.player().name()), Color::White);
                    }
                }
                self.set_open(false);
            }
            self.invalidate();
            true
        }
        else if inputs.key_was_pressed(Key::T) {
            self.set_open(true);
            self.current_message.text_mut().clear();
            self.invalidate();
            true
        }
        else if inputs.key_was_pressed(Key::Slash) {
            self.set_open(true);
            self.current_message.text_mut().clear();
            self.current_message.text_mut().push('/');
            self.invalidate();
            true
        }
        else {
            false
        }
    }

    pub fn add_plain_message(&mut self, text: String, color: Color) {
        self.add_message(ChatMessage::new(
            TextLine::new(
                Vector([0.0, 1.0]),
                color.into(),
                TextBackground::Rectangle {
                    color: Color::Black.with_alpha(self.background_opacity),
                    margin: Vector([1.0, 0.0]),
                },
                text,
            ),
            Instant::now(),
        ));
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        while self.message_history.len() >= self.max_history {
            self.message_history.pop_front();
        }
        self.message_history.push_back(message);
    }

    pub fn render(&mut self, render_pass: &mut wgpu::RenderPass, assets: &mut AssetPool) {
        if self.text_layer.is_empty() {
            if self.is_open {
                self.current_message.append_to_mesh(
                    self.text_layer.mesh_mut(),
                    self.offset,
                    assets,
                );
            }

            let mut offset = self.offset + Vector([0.0, -4.0]);
            for message in self.message_history.iter_mut().rev() {
                offset.set_y(offset.y() - self.line_spacing);
                message.line_mut().append_to_mesh(
                    self.text_layer.mesh_mut(),
                    offset,
                    assets,
                );
            }

            self.text_layer.upload_buffers();
        }

        assets.gui_shaders().set_uniform("anchor", &self.anchor);
        assets.gui_texture().bind();
        self.text_layer.render(render_pass);
    }
}

pub struct ChatMessage {
    line: TextLine,
    time_received: Instant,
}

impl ChatMessage {
    pub fn new(line: TextLine, time_received: Instant) -> Self {
        Self {
            line,
            time_received,
        }
    }

    pub fn line(&self) -> &TextLine {
        &self.line
    }

    pub fn line_mut(&mut self) -> &mut TextLine {
        &mut self.line
    }

    pub fn time_received(&self) -> Instant {
        self.time_received
    }
}
