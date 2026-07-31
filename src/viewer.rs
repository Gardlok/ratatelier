use std::{
    io::{self, stdout},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Sparkline, Widget, Wrap},
};

use crate::{
    export,
    model::{AnimationFrame, Canvas, ComponentScene, Project, RectSpec, WidgetKind},
    ui,
};

const TICK_RATE: Duration = Duration::from_millis(16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewerSurface {
    #[default]
    Artwork,
    Components,
}

impl ViewerSurface {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Artwork => "artwork",
            Self::Components => "components",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ViewerOptions {
    pub surface: ViewerSurface,
    pub loop_animation: bool,
}

#[derive(Debug)]
struct ViewerState {
    project: Project,
    options: ViewerOptions,
    active_frame: usize,
    playing: bool,
    completed: bool,
    running: bool,
    elapsed_in_frame: Duration,
    last_tick: Instant,
}

impl ViewerState {
    fn new(project: Project, options: ViewerOptions) -> Self {
        let playing = project.frames.len() > 1;
        Self {
            project,
            options,
            active_frame: 0,
            playing,
            completed: false,
            running: true,
            elapsed_in_frame: Duration::ZERO,
            last_tick: Instant::now(),
        }
    }

    fn frame(&self) -> &AnimationFrame {
        &self.project.frames[self.active_frame]
    }

    fn tick(&mut self) {
        let now = Instant::now();
        let delta = now.saturating_duration_since(self.last_tick);
        self.last_tick = now;
        self.advance(delta);
    }

    fn advance(&mut self, delta: Duration) {
        if !self.playing || self.project.frames.len() <= 1 {
            return;
        }
        self.elapsed_in_frame += delta;
        loop {
            let duration = Duration::from_millis(self.frame().duration_ms.max(1));
            if self.elapsed_in_frame < duration {
                break;
            }
            self.elapsed_in_frame -= duration;
            if self.active_frame + 1 < self.project.frames.len() {
                self.active_frame += 1;
            } else if self.options.loop_animation {
                self.active_frame = 0;
            } else {
                self.playing = false;
                self.completed = true;
                self.elapsed_in_frame = Duration::ZERO;
                break;
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q' | 'Q') => self.running = false,
            KeyCode::Char('r' | 'R') => self.replay(),
            KeyCode::Char(' ') if self.project.frames.len() > 1 => {
                if self.completed {
                    self.replay();
                } else {
                    self.playing = !self.playing;
                    self.last_tick = Instant::now();
                }
            }
            _ => {}
        }
    }

    fn replay(&mut self) {
        self.active_frame = 0;
        self.completed = false;
        self.elapsed_in_frame = Duration::ZERO;
        self.playing = self.project.frames.len() > 1;
        self.last_tick = Instant::now();
    }
}

pub fn run(project: Project, options: ViewerOptions) -> io::Result<()> {
    enable_raw_mode()?;
    let mut output = stdout();
    execute!(output, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(output);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = run_loop(&mut terminal, ViewerState::new(project, options));

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut viewer: ViewerState,
) -> io::Result<()> {
    while viewer.running {
        terminal.draw(|frame| draw(frame, &viewer))?;
        if event::poll(TICK_RATE)? {
            match event::read()? {
                Event::Key(key)
                    if matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) =>
                {
                    viewer.handle_key(key);
                }
                Event::Resize(_, _) | Event::FocusGained | Event::FocusLost => {}
                _ => {}
            }
        }
        viewer.tick();
    }
    Ok(())
}

fn draw(frame: &mut Frame<'_>, viewer: &ViewerState) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());
    match viewer.options.surface {
        ViewerSurface::Artwork => frame.render_widget(
            ArtworkView {
                canvas: &viewer.frame().canvas,
            },
            areas[0],
        ),
        ViewerSurface::Components => draw_components(frame, viewer, areas[0]),
    }
    draw_footer(frame, viewer, areas[1]);
}

struct ArtworkView<'a> {
    canvas: &'a Canvas,
}

impl Widget for ArtworkView<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        let width = self.canvas.width.min(area.width);
        let height = self.canvas.height.min(area.height);
        let source_x = self.canvas.width.saturating_sub(width) / 2;
        let source_y = self.canvas.height.saturating_sub(height) / 2;
        let target_x = area.x + area.width.saturating_sub(width) / 2;
        let target_y = area.y + area.height.saturating_sub(height) / 2;

        for y in 0..height {
            for x in 0..width {
                let point = crate::model::Point::new(source_x + x, source_y + y);
                let Some(cell) = self.canvas.composite_cell(point) else {
                    continue;
                };
                buffer[(target_x + x, target_y + y)]
                    .set_symbol(if cell.transparent { " " } else { &cell.glyph })
                    .set_style(ui::cell_style(&cell.style));
            }
        }
    }
}

fn draw_components(frame: &mut Frame<'_>, viewer: &ViewerState, area: Rect) {
    let scene = &viewer.project.components;
    let target = natural_area(scene.width, scene.height, area);
    let state = viewer.frame().widget_state;
    for widget in &scene.widgets {
        let widget_area = scale_rect(widget.rect, target, scene);
        if widget_area.width == 0 || widget_area.height == 0 {
            continue;
        }
        let style = ui::cell_style(widget.styles.for_state(state));
        let block = Block::default()
            .borders(Borders::ALL)
            .title(widget.title.as_str())
            .border_style(style);
        match widget.kind {
            WidgetKind::Block => frame.render_widget(block.style(style), widget_area),
            WidgetKind::Paragraph => frame.render_widget(
                Paragraph::new(widget.text.as_str())
                    .block(block)
                    .style(style)
                    .wrap(Wrap { trim: false }),
                widget_area,
            ),
            WidgetKind::Gauge => frame.render_widget(
                Gauge::default()
                    .block(block)
                    .ratio(f64::from(widget.value.min(100)) / 100.0)
                    .gauge_style(style),
                widget_area,
            ),
            WidgetKind::List => {
                let items: Vec<ListItem<'_>> = widget.text.lines().map(ListItem::new).collect();
                frame.render_widget(List::new(items).block(block).style(style), widget_area);
            }
            WidgetKind::Sparkline => {
                let data = export::parse_sparkline(&widget.text);
                frame.render_widget(
                    Sparkline::default().block(block).data(&data).style(style),
                    widget_area,
                );
            }
        }
    }
}

fn natural_area(source_width: u16, source_height: u16, area: Rect) -> Rect {
    let width = source_width.min(area.width);
    let height = source_height.min(area.height);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}

fn scale_rect(rect: RectSpec, area: Rect, scene: &ComponentScene) -> Rect {
    if scene.width == 0 || scene.height == 0 || area.width == 0 || area.height == 0 {
        return Rect::default();
    }
    let x = area.x + rect.x.saturating_mul(area.width) / scene.width;
    let y = area.y + rect.y.saturating_mul(area.height) / scene.height;
    let width = rect
        .width
        .saturating_mul(area.width)
        .div_ceil(scene.width)
        .max(1)
        .min(area.right().saturating_sub(x));
    let height = rect
        .height
        .saturating_mul(area.height)
        .div_ceil(scene.height)
        .max(1)
        .min(area.bottom().saturating_sub(y));
    Rect::new(x, y, width, height)
}

fn draw_footer(frame: &mut Frame<'_>, viewer: &ViewerState, area: Rect) {
    let playback = if viewer.project.frames.len() <= 1 {
        "STATIC"
    } else if viewer.playing {
        "PLAYING"
    } else if viewer.completed {
        "COMPLETE"
    } else {
        "PAUSED"
    };
    let playback_color = if viewer.playing {
        Color::LightGreen
    } else {
        Color::DarkGray
    };
    let controls = if viewer.project.frames.len() > 1 {
        "Space pause/resume · r replay · q close"
    } else {
        "q close"
    };
    let line = Line::from(vec![
        Span::styled(
            format!(" {playback} "),
            Style::default()
                .fg(Color::Black)
                .bg(playback_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(
            " {} · {} · {}/{} · {controls}",
            viewer.project.name,
            viewer.options.surface.label(),
            viewer.active_frame + 1,
            viewer.project.frames.len()
        )),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn animated_project() -> Project {
        let mut project = Project::new("viewer", 2, 1);
        project.frame_mut().duration_ms = 10;
        project.add_frame();
        project.frame_mut().duration_ms = 20;
        project
    }

    #[test]
    fn animation_starts_playing_and_stops_after_one_pass() {
        let mut viewer = ViewerState::new(animated_project(), ViewerOptions::default());
        assert!(viewer.playing);
        viewer.advance(Duration::from_millis(10));
        assert_eq!(viewer.active_frame, 1);
        viewer.advance(Duration::from_millis(20));
        assert!(!viewer.playing);
        assert!(viewer.completed);
        assert_eq!(viewer.active_frame, 1);
    }

    #[test]
    fn replay_returns_to_the_first_frame() {
        let mut viewer = ViewerState::new(animated_project(), ViewerOptions::default());
        viewer.advance(Duration::from_millis(30));
        viewer.replay();
        assert_eq!(viewer.active_frame, 0);
        assert!(viewer.playing);
        assert!(!viewer.completed);
    }

    #[test]
    fn looping_animation_wraps() {
        let mut viewer = ViewerState::new(
            animated_project(),
            ViewerOptions {
                loop_animation: true,
                ..ViewerOptions::default()
            },
        );
        viewer.advance(Duration::from_millis(30));
        assert_eq!(viewer.active_frame, 0);
        assert!(viewer.playing);
    }
}
