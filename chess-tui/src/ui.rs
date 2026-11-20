//! Drawing: board, side panel (status + move list), status bar, input line, popups.

use crate::app::{App, MessageKind, Mode};
use crate::input::Keymap;
use chess_core::{AnyGame, Color, Piece, PieceKind, Square};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color as Tui, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

const LIGHT: Tui = Tui::Rgb(205, 170, 125);
const DARK: Tui = Tui::Rgb(140, 100, 65);
const CURSOR: Tui = Tui::Rgb(70, 130, 200);
const SELECTED: Tui = Tui::Rgb(215, 195, 60);
const TARGET_LIGHT: Tui = Tui::Rgb(150, 190, 110);
const TARGET_DARK: Tui = Tui::Rgb(105, 150, 75);
const LAST_LIGHT: Tui = Tui::Rgb(220, 205, 120);
const LAST_DARK: Tui = Tui::Rgb(175, 155, 75);
const CHECK: Tui = Tui::Rgb(210, 60, 60);
const WHITE_PIECE: Tui = Tui::Rgb(255, 255, 255);
const BLACK_PIECE: Tui = Tui::Rgb(0, 0, 0);

/// Width of the side panel.
const SIDE_WIDTH: u16 = 30;

/// Draws the whole screen.
pub fn draw(frame: &mut Frame, app: &App, keymap: &Keymap) {
    let [main, status, input] = Layout::vertical([
        Constraint::Min(10),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let (cw, ch) = cell_size(main);
    let board_w = cw * 8 + 2 + 2; // squares + rank labels + borders
    let [board_area, side_area] =
        Layout::horizontal([Constraint::Length(board_w), Constraint::Min(SIDE_WIDTH)]).areas(main);

    draw_board(frame, board_area, app, cw, ch);
    draw_side(frame, side_area, app);
    draw_status(frame, status, app);
    draw_input(frame, input, app);

    match app.mode {
        Mode::Help => draw_help(frame, keymap),
        Mode::Promotion { .. } => draw_promotion(frame),
        _ => {}
    }
}

/// Largest square size (width, height in cells) that fits; squares look
/// roughly square because terminal cells are about twice as tall as wide.
fn cell_size(area: Rect) -> (u16, u16) {
    [(7, 3), (5, 2), (3, 1)]
        .into_iter()
        .find(|&(w, h)| area.height >= h * 8 + 3 && area.width >= w * 8 + 4 + SIDE_WIDTH)
        .unwrap_or((3, 1))
}

fn glyph(piece: Piece, ascii: bool) -> char {
    if ascii {
        piece.to_fen_char()
    } else {
        // Solid glyphs for both sides; colour tells them apart (more legible than outlines).
        Piece::new(Color::Black, piece.kind).to_unicode()
    }
}

fn draw_board(frame: &mut Frame, area: Rect, app: &App, cw: u16, ch: u16) {
    let block = Block::bordered().title(" phantom-chess ");
    let Some(game) = app.game() else {
        frame.render_widget(block, area);
        return;
    };
    let pos = game.position();
    let targets = app.targets();
    let last = app.last_move();
    let check_sq = (!game.is_over() && pos.in_check())
        .then(|| pos.board().find_king(pos.side_to_move()))
        .flatten();

    let ranks: Vec<u8> = if app.flipped {
        (0..8).collect()
    } else {
        (0..8).rev().collect()
    };
    let files: Vec<u8> = if app.flipped {
        (0..8).rev().collect()
    } else {
        (0..8).collect()
    };
    let w = usize::from(cw);

    let mut lines = Vec::new();
    for &rank in &ranks {
        for row in 0..ch {
            let mid = row == ch / 2;
            let label = if mid {
                format!("{} ", rank + 1)
            } else {
                "  ".into()
            };
            let mut spans = vec![Span::styled(label, Style::new().fg(Tui::Gray))];
            for &file in &files {
                let Some(sq) = Square::new(file, rank) else {
                    continue;
                };
                let light = (file + rank) % 2 == 1;
                let piece = pos.piece_at(sq);
                let bg = if sq == app.cursor {
                    CURSOR
                } else if Some(sq) == app.selected {
                    SELECTED
                } else if Some(sq) == check_sq {
                    CHECK
                } else if targets.contains(&sq) {
                    if light { TARGET_LIGHT } else { TARGET_DARK }
                } else if last.is_some_and(|m| m.from == sq || m.to == sq) {
                    if light { LAST_LIGHT } else { LAST_DARK }
                } else if light {
                    LIGHT
                } else {
                    DARK
                };
                let (text, fg) = match (mid, piece) {
                    (true, Some(p)) => (
                        glyph(p, app.ascii).to_string(),
                        if p.color == Color::White {
                            WHITE_PIECE
                        } else {
                            BLACK_PIECE
                        },
                    ),
                    (true, None) if targets.contains(&sq) => ("·".into(), BLACK_PIECE),
                    _ => (String::new(), BLACK_PIECE),
                };
                spans.push(Span::styled(
                    format!("{text:^w$}"),
                    Style::new().bg(bg).fg(fg).add_modifier(Modifier::BOLD),
                ));
            }
            lines.push(Line::from(spans));
        }
    }
    let mut footer = vec![Span::raw("  ")];
    for &file in &files {
        let letter = char::from(b'a' + file);
        footer.push(Span::styled(
            format!("{letter:^w$}"),
            Style::new().fg(Tui::Gray),
        ));
    }
    lines.push(Line::from(footer));

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_side(frame: &mut Frame, area: Rect, app: &App) {
    let [info_area, moves_area] =
        Layout::vertical([Constraint::Length(6), Constraint::Min(3)]).areas(area);
    let Some(game) = app.game() else { return };

    let mut info = Vec::new();
    match game.result() {
        Some(result) => {
            info.push(Line::styled(
                "Game over",
                Style::new().add_modifier(Modifier::BOLD),
            ));
            info.push(Line::from(result.to_string()));
            info.push(Line::styled(
                "n: new game  u: undo",
                Style::new().fg(Tui::Gray),
            ));
        }
        None => {
            let side = game.position().side_to_move();
            info.push(Line::from(vec![
                Span::raw("Turn: "),
                Span::styled(side.to_string(), Style::new().add_modifier(Modifier::BOLD)),
            ]));
            if game.position().in_check() {
                info.push(Line::styled(
                    "CHECK!",
                    Style::new().fg(CHECK).add_modifier(Modifier::BOLD),
                ));
            }
            info.push(Line::from(format!(
                "Move {}  (50-move clock {})",
                game.position().fullmove_number(),
                game.position().halfmove_clock()
            )));
        }
    }
    if app.flipped {
        info.push(Line::styled("(board flipped)", Style::new().fg(Tui::Gray)));
    }
    frame.render_widget(
        Paragraph::new(info).block(Block::bordered().title(" Status ")),
        info_area,
    );

    let rows = move_rows(game);
    let visible = usize::from(moves_area.height.saturating_sub(2));
    let skip = rows.len().saturating_sub(visible);
    let lines: Vec<Line> = rows.into_iter().skip(skip).map(Line::from).collect();
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(" Moves ")),
        moves_area,
    );
}

/// Move list as numbered rows: `1. e2e4  e7e5`.
fn move_rows(game: &AnyGame) -> Vec<String> {
    let history = game.history();
    let now = game.position().side_to_move();
    // Who made the first recorded move (a loaded FEN may start with Black).
    let first = if history.len().is_multiple_of(2) {
        now
    } else {
        now.opposite()
    };
    let mut cells: Vec<String> = history.iter().map(ToString::to_string).collect();
    if first == Color::Black {
        cells.insert(0, "...".into());
    }
    cells
        .chunks(2)
        .enumerate()
        .map(|(i, pair)| format!("{:>3}. {}", i + 1, pair.join("  ")))
        .collect()
}

fn draw_status(frame: &mut Frame, area: Rect, app: &App) {
    let line = match &app.message {
        Some(msg) => {
            let color = match msg.kind {
                MessageKind::Info => Tui::Green,
                MessageKind::Error => Tui::LightRed,
            };
            Line::styled(msg.text.clone(), Style::new().fg(color))
        }
        None => Line::default(),
    };
    frame.render_widget(Paragraph::new(line), area);
}

fn draw_input(frame: &mut Frame, area: Rect, app: &App) {
    let prompt = match app.mode {
        Mode::Command => Some("move> "),
        Mode::SaveName => Some("save as> "),
        Mode::LoadName => Some("load> "),
        _ => None,
    };
    let line = match prompt {
        Some(p) => Line::from(vec![
            Span::styled(p, Style::new().fg(Tui::Yellow)),
            Span::raw(app.input.clone()),
            Span::styled("█", Style::new().fg(Tui::Gray)),
        ]),
        None => Line::styled(
            "arrows/hjkl move · Enter select · : type move · u undo · ? help · q quit",
            Style::new().fg(Tui::DarkGray),
        ),
    };
    frame.render_widget(Paragraph::new(line), area);
}

/// A `w`×`h` rectangle centred in `area` (clamped to fit).
fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    }
}

fn draw_help(frame: &mut Frame, keymap: &Keymap) {
    let mut lines: Vec<Line> = keymap
        .bindings()
        .iter()
        .map(|b| {
            Line::from(vec![
                Span::styled(
                    format!("{:<14}", b.label),
                    Style::new().fg(Tui::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::raw(b.description),
            ])
        })
        .collect();
    lines.push(Line::default());
    lines.push(Line::styled(
        "In prompts: Enter confirms, Esc cancels. Ctrl-C quits.",
        Style::new().fg(Tui::Gray),
    ));
    lines.push(Line::styled(
        "Press any key to close.",
        Style::new().fg(Tui::Gray),
    ));

    let height = u16::try_from(lines.len())
        .unwrap_or(u16::MAX)
        .saturating_add(2);
    let area = centered(frame.area(), 56, height);
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::bordered().title(" Keys "))
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn draw_promotion(frame: &mut Frame) {
    let options: Vec<Span> = PieceKind::PROMOTIONS
        .iter()
        .flat_map(|&k| {
            [
                Span::styled(
                    format!(" {} ", k.to_char()),
                    Style::new().fg(Tui::Yellow).add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("{k}  ")),
            ]
        })
        .collect();
    let lines = vec![
        Line::from(options),
        Line::styled("Esc to cancel", Style::new().fg(Tui::Gray)),
    ];
    let area = centered(frame.area(), 48, 4);
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(" Promote to ")),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn render(app: &App, w: u16, h: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
        terminal.draw(|f| draw(f, app, &Keymap::new())).unwrap();
        let buffer = terminal.backend().buffer();
        buffer
            .content()
            .chunks(usize::from(w))
            .map(|row| row.iter().map(|c| c.symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn renders_at_small_and_large_sizes() {
        let mut app = App::new();
        app.ascii = true;
        for (w, h) in [(60, 14), (100, 30), (160, 50)] {
            let screen = render(&app, w, h);
            assert!(screen.contains("phantom-chess"), "{w}x{h}");
            assert!(screen.contains("Turn: White"), "{w}x{h}");
        }
    }

    #[test]
    fn tiny_terminal_does_not_panic() {
        let app = App::new();
        render(&app, 10, 5);
        render(&app, 1, 1);
    }

    #[test]
    fn help_and_promotion_popups_render() {
        let mut app = App::new();
        app.show_help();
        assert!(render(&app, 100, 30).contains("flip board"));
        app.mode = Mode::Promotion {
            from: Square::try_from("a7").unwrap(),
            to: Square::try_from("a8").unwrap(),
        };
        assert!(render(&app, 100, 30).contains("Promote to"));
    }

    #[test]
    fn move_list_numbers_pairs() {
        let mut app = App::new();
        for m in ["e2e4", "e7e5", "g1f3"] {
            app.try_play(m.try_into().unwrap());
        }
        assert_eq!(
            move_rows(app.game().unwrap()),
            vec!["  1. e2e4  e7e5", "  2. g1f3"]
        );
    }
}
