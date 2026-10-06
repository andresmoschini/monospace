//! Q6b — The SAME picture in plain ratatui. No tuirealm at all.
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use ratatui::Terminal;

#[derive(Default)]
struct S {
    menu: bool,
    popup: bool,
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
    // render off-screen so the spike can print it; same draw() as a real app
    let backend = ratatui::backend::TestBackend::new(40, 10);
    let mut t = Terminal::new(backend).unwrap();
    let s = S { menu: true, popup: true };
    let fr = t.draw(|f| draw(f, &s)).unwrap();
    print!("{}", tuirealm::testing::buffer_to_string(&fr.buffer));
}
