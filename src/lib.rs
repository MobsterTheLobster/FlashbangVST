#[macro_use]
extern crate vst;

use vst::api::{Events};
use vst::editor::Editor;
use vst::buffer::AudioBuffer;
use vst::event::Event;
use vst::plugin::{Category, HostCallback, Info, Plugin, PluginParameters};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use tuix::Application;
use tuix::*;
use vst::util::AtomicFloat;

use std::f64::consts::PI;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering, AtomicBool};

static THEME: &str = include_str!("theme.css");

const W_WIDTH: usize = 1000;
const W_HEIGHT: usize = 1000;

struct FrameCallback(Entity, Arc<FlashbangParameters>, Entity, Entity);

struct KnobParentWidget {
    params: Arc<FlashbangParameters>,
    red_knob: Entity,
    green_knob: Entity,
    blue_knob: Entity,
    frequency_knob: Entity
}

impl KnobParentWidget {
    pub fn new(params: Arc<FlashbangParameters>) -> Self {
        KnobParentWidget {
            params: params.clone(),
            red_knob: Entity::null(),
            green_knob: Entity::null(),
            blue_knob: Entity::null(),
            frequency_knob: Entity::null()
        }
    }
}

impl Widget for KnobParentWidget {
    type Ret = Entity;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        state.set_focus(entity);
        entity.set_visibility(state, Visibility::Invisible);

        let red = self.params.red.load(Ordering::Relaxed);
        self.red_knob = ValueKnob::new("Red", red as f32, 0.0, 255.0)
            .build(state, entity, |builder| {builder});

        let green = self.params.green.load(Ordering::Relaxed);
        self.green_knob = ValueKnob::new("Green", green as f32, 0.0, 255.0)
            .build(state, entity, |builder| {builder});
            
        let blue = self.params.blue.load(Ordering::Relaxed);
        self.blue_knob = ValueKnob::new("Blue", blue as f32, 0.0, 255.0)
            .build(state, entity, |builder| {builder});
        
        let frequency = self.params.frequency.get();
        self.frequency_knob = ValueKnob::new("Frequency", frequency as f32, 0.0, 10000.0)
            .build(state, entity, |builder| {builder});
        
        entity
    }

    fn on_event(&mut self, _state: &mut State, _entity: Entity, event: &mut tuix::Event) {
        if let Some(slider_event) = event.message.downcast::<SliderEvent>() {
            match slider_event {
                SliderEvent::ValueChanged(val) => {
                    if event.target == self.red_knob {
                        self.params.red.store(*val as u8, Ordering::Relaxed);
                    }
                    if event.target == self.green_knob {
                        self.params.green.store(*val as u8, Ordering::Relaxed);
                    }
                    if event.target == self.blue_knob {
                        self.params.blue.store(*val as u8, Ordering::Relaxed);
                    }
                    if event.target == self.frequency_knob {
                        self.params.frequency.set(*val);
                    }
                }
                _ => {}
            }
        }
    }

    /*fn on_draw(&mut self, state: &mut State, _entity: Entity, _canvas: &mut widget::Canvas) {
        
    }*/
}

impl Widget for FrameCallback {
    type Ret = Entity;
    fn on_build(&mut self, state: &mut State, entity: Entity) -> Self::Ret {
        let params = self.1.clone();
        
        self.2 = KnobParentWidget::new(params.clone()).build(state, entity, |builder| {builder});
        let parent_widget = self.2;
        self.3 = Button::with_label("Settings").on_press(move |_button, state, _id| {
            if state.data.get_visibility(parent_widget) == Visibility::Visible {
                parent_widget.set_visibility(state, Visibility::Invisible);
            } else {
                parent_widget.set_visibility(state, Visibility::Visible);
            }
        })
        .build(state, entity, |builder| {
            builder
                .set_position_type(PositionType::SelfDirected)
                .set_left(Stretch(1.0))
                .set_top(Pixels(0.0))
                .set_bottom(Stretch(1.0))
                .set_right(Pixels(0.0))
                .set_width(Pixels(100.0))
        });
        entity
    }
    fn on_draw(&mut self, state: &mut State, _entity: Entity, _canvas: &mut widget::Canvas) {
        // do stuff
        
        self.0.set_background_color(state, Color::rgb(
            (self.1.red.load(Ordering::Relaxed) as f32 * self.1.brightness.get()) as u8,
            (self.1.green.load(Ordering::Relaxed) as f32 * self.1.brightness.get()) as u8,
            (self.1.blue.load(Ordering::Relaxed) as f32 * self.1.brightness.get()) as u8
        ));
    }
}

#[derive(Clone)]
#[derive(Default)]
struct FlashbangEditor {
    open: bool,
    params: Arc<FlashbangParameters>
}

struct FlashbangParameters {
    brightness: AtomicFloat,
    playing: AtomicBool,
    frequency: AtomicFloat,
    red: AtomicU8,
    green: AtomicU8,
    blue: AtomicU8
}

impl Default for FlashbangParameters {
    fn default() -> FlashbangParameters {
        FlashbangParameters {
            brightness: AtomicFloat::new(0.),
            playing: AtomicBool::new(false),
            frequency: AtomicFloat::new(5000.),
            red: AtomicU8::new(255),
            green: AtomicU8::new(255),
            blue: AtomicU8::new(255),
        }
    }
}

impl PluginParameters for FlashbangParameters {
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            0 => self.brightness.get().into(),
            1 => self.red.load(Ordering::Relaxed).into(),
            2 => self.green.load(Ordering::Relaxed).into(),
            3 => self.blue.load(Ordering::Relaxed).into(),
            4 => self.frequency.get().into(),
            _ => 0.0
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            0 => format!("{}", (self.brightness.get())),
            1 => format!("{}", (self.red.load(Ordering::Relaxed))),
            2 => format!("{}", (self.green.load(Ordering::Relaxed))),
            3 => format!("{}", (self.blue.load(Ordering::Relaxed))),
            4 => format!("{}", (self.frequency.get())),
            _ => "".to_string()
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            0 => "Brightness",
            1 => "Red",
            2 => "Green",
            3 => "Blue",
            4 => "Frequency",
            _ => ""
        }.to_string()
    }

    fn set_parameter(&self, index: i32, value: f32) {
        match index {
            0 => self.brightness.set(value),
            1 => self.red.store(value as u8, Ordering::Relaxed),
            2 => self.green.store(value as u8, Ordering::Relaxed),
            3 => self.blue.store(value as u8, Ordering::Relaxed),
            4 => self.frequency.set(value),
            _ => ()
        }
    }
}

impl Editor for FlashbangEditor {
    fn position(&self) -> (i32, i32) {
        (0,0)
    }

    fn size(&self) -> (i32, i32) {
        (W_WIDTH as i32, W_HEIGHT as i32)
    }

    fn open(&mut self, parent: *mut std::ffi::c_void) -> bool {
        if self.open {
            return false;
        }

        self.open = true;

        let window_description = WindowDescription::new().with_inner_size(W_WIDTH as u32, W_HEIGHT as u32).with_title("Flashbang");
        let params = Arc::clone(&self.params);
        Application::new(window_description, move |state, window| {
            state.add_theme(THEME);
            FrameCallback(window.entity(), params, Entity::null(), Entity::null()).build(state, window.entity(), |builder| builder);
        }).open_parented(&VstParent(parent));
        
        true
    }

    fn is_open(&mut self) -> bool {
        self.open
    }

    fn close(&mut self) {
        self.open = false;
    }
}

#[derive(Default)]
struct Flashbang {
    sample_rate: f64,
    note_duration: f64,
    time: f64,
    phase: f64,
    playing: bool,
    editor: FlashbangEditor,
}

impl Flashbang {
    fn time_per_sample(&self) -> f64 {
        1.0 / self.sample_rate
    }

    fn process_midi_event(&mut self, data: [u8; 3]) {
        match data[0] {
            144 => self.play(),
            _ => (),
        }
    }

    fn play(&mut self) {
        self.note_duration = 0.0;
        self.playing = true;
    }
}

pub const TAU: f64 = PI * 2.0;

impl Plugin for Flashbang {
    fn new(_host: HostCallback) -> Self {
        let params = Arc::new(FlashbangParameters::default());
        Flashbang {
            sample_rate: 44100.0,
            note_duration: 0.0,
            time: 0.0,
            phase: 0.0,
            playing: false,
            editor: FlashbangEditor {
                open: false,
                params: params
            }
        }
    }

    fn get_info(&self) -> Info {
        Info { 
            name: "Flashbang".to_string(),
            vendor: "Mobster the Lobster#6955".to_string(),
            inputs: 2,
            outputs: 2,
            unique_id: 6969,
            category: Category::Synth,
            initial_delay: 0,
            ..Info::default()
        }
    }

    fn get_editor(&mut self) -> Option<Box<(dyn Editor + 'static)>> {
        if let editor = self.editor.clone() {
            Some(Box::new(editor) as Box<dyn Editor>)
        } else {
            None
        }
    }

    #[allow(clippy::clippy::single_match)]
    fn process_events(&mut self, events: &Events) {
        for event in events.events() {
            match event {
                Event::Midi(ev) => self.process_midi_event(ev.data),
                _ => ()
            }
        }
    }

    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = f64::from(rate);
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let samples = buffer.samples();
        let (_, mut outputs) = buffer.split();
        let output_count = outputs.len();
        let per_sample = self.time_per_sample();
        let mut output_sample;

        for sample_idx in 0..samples {
            let time = self.time;
            let note_duration = self.note_duration;
            let freq = self.editor.params.frequency.get() as f64;
            self.editor.params.playing.store(self.playing, Ordering::Relaxed);
            if self.playing {
                let signal = (self.phase * TAU).sin();

                let attack = 0.2;
                let decay = 10.0;
                let alpha = if note_duration < attack {
                    note_duration / attack
                } else if note_duration < (decay+attack) {
                    -((note_duration - attack) / decay) + (1.0)
                } else {
                    0.0
                };
                if note_duration > (attack+decay) {
                    self.playing = false
                } else {
                    let brightness: f32 = alpha as f32;
                    self.editor.params.brightness.set(brightness);
                }

                output_sample = (signal * alpha) as f32;

                self.time += per_sample;
                self.phase = (self.phase + freq as f64 / self.sample_rate).fract();
                self.note_duration += per_sample;
            } else {
                output_sample = 0.0;
            }
            for buf_idx in 0..output_count {
                let buff = outputs.get_mut(buf_idx);
                buff[sample_idx] = output_sample;
            }
        }
    }
}

struct VstParent(*mut ::std::ffi::c_void);

unsafe impl HasRawWindowHandle for VstParent {
    fn raw_window_handle(&self) -> RawWindowHandle {
        use raw_window_handle::windows::WindowsHandle;

        RawWindowHandle::Windows(WindowsHandle {
            hwnd: self.0,
            ..WindowsHandle::empty()
        })
    }
}

plugin_main!(Flashbang);