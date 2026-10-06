//! Q6b' — plain ratatui doing the SAME job as q6_tuirealm.rs, including the
//! event loop, the Msg enum, the update function and the mouse hit-test.
//! This is the fair comparison: nothing is left out of the ratatui side.
use std::io::IsTerminal;

use ratatui::crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind};
use ratatui::crossterm::execute;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::{DefaultTerminal, Frame};

#[derive(PartialEq, Debug, Clone)]
enum Msg {
    OpenMenu,
    CloseMenu,
    OpenPopup,
    CloseClicked,
    Bump,
}

#[derive(Default)]
struct S {
    menu: bool,
    popup: bool,
    n: i64,
}

/// Absolute rect of the Close button, needed by the hit-test.
const CLOSE: Rect = Rect::new(10, 4, 10, 3);

fn update(s: &mut S, m: Msg) {
    match m {
        Msg::OpenMenu => s.menu = true,
        Msg::CloseMenu => s.menu = false,
        Msg::OpenPopup => s.popup = true,
        Msg::CloseClicked => s.popup = false,
        Msg::Bump => s.n += 1,
    }
}

fn handle_key(s: &mut S, code: KeyCode) -> Option<Msg> {
    Some(match code {
        KeyCode::F(1) => Msg::OpenMenu,
        KeyCode::F(2) => Msg::OpenPopup,
        KeyCode::Esc => Msg::CloseMenu,
        KeyCode::Char('q') => Msg::Bump,
        _ => return None,
    })
}

fn handle_mouse(m: ratatui::crossterm::event::MouseEvent) -> Option<Msg> {
    if let MouseEventKind::Down(MouseButton::Left) = m.kind {
        let c = m.column;
        let r = m.row;
        if r >= CLOSE.y && r < CLOSE.y + CLOSE.height && c >= CLOSE.x && c < CLOSE.x + CLOSE.width {
            return Some(Msg::CloseClicked);
        }
    }
    None
}

fn draw(f: &mut Frame, s: &S) {
    let [bar, mid, st] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(f.area());

    f.render_widget(
        Paragraph::new(vec![
            Line::from("File").style(Style::default().fg(Color::Cyan)),
            Line::from("  Edit  View"),
        ]),
        bar,
    );

    if s.menu {
        f.render_widget(
            Paragraph::new(" Open\n Save\n Exit").block(Block::default().borders(Borders::ALL)),
            Rect::new(bar.x, mid.y, 8, 4).intersection(mid),
        );
    }
    f.render_widget(Paragraph::new(" ASCII canvas ┌─┐ here"), mid);

    if s.popup {
        let d = Rect::new(f.area().x + 4, f.area().y + 1, 24, 6);
        f.render_widget(Clear, d);
        f.render_widget(
            Paragraph::new(" Discard changes?\n\n   Close   ")
                .block(Block::default().borders(Borders::ALL).title(" Confirm ")),
            d,
        );
    }
    f.render_widget(Paragraph::new(" F1 menu  F2 dialog  q quit"), st);
}

fn main() {
    let mut s = S { menu: true, popup: true, n: 0 };
    if std::io::stdout().is_terminal() {
        let mut t = ratatui::init::init();
        t.backend_mut();
        execute!(std::io::stdout(), EnableMouseCapture).unwrap();
        loop {
            t.draw(|f| draw(f, &s)).unwrap();
            match ratatui::crossterm::event::read() {
                Ok(Event::Key(k)) => {
                    if let Some(m) = handle_key(&mut s, k.code) {
                        update(&mut s, m);
                    }
                }
                Ok(Event::Mouse(m)) => {
                    if let Some(m) = handle_mouse(m) {
                        update(&mut s, m);
                    }
                }
                _ => {}
            }
        }
    }
    // headless: print the same picture
    let backend = ratatui::backend::TestBackend::new(40, 10);
    let mut t = ratatui::Terminal::new(backend).unwrap();
    let fr = t.draw(|f| draw(f, &s)).unwrap();
    print!("{}", tuirealm::testing::buffer_to_string(&fr.buffer));
}
