// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

mod adapters;
mod app;
mod architecture_audit;
mod atlas;
mod awareness;
mod benchmark;
mod bridge;
mod conductor;
mod context;
mod core;
mod decision;
mod diagnostics;
mod domain;
mod evaluation;
mod file_plan;
mod forge;
mod gateway;
mod goal;
mod guard;
mod harbor;
mod hardening;
mod health;
mod http_transport;
mod intelligence;
mod intent;
mod journal;
mod knowledge;
mod lens;
mod memory;
mod model;
mod operator;
mod patch;
mod persistence;
mod profile;
mod project_dna;
mod providers;
mod recovery;
mod reflection;
mod regression;
mod release;
mod request_control;
mod resource;
mod rollback;
mod sandbox;
mod self_verification;
mod streaming;
mod telemetry;
mod text_patch;
mod transaction;
mod ui;
mod workspace;
mod workspace_diff;
mod workspace_index;
mod workspace_io;
mod workspace_lock;
mod workspace_ops;

use std::{error::Error, io, time::Duration};

use app::App;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::demo();
    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result?;
    Ok(())
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key);
                }
            }
        }
    }

    Ok(())
}
