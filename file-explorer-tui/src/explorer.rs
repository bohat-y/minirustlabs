use anyhow::Result;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::{Attribute, Color, SetAttribute, SetForegroundColor, ResetColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{stdout, Write},
    path::PathBuf,
};

use crate::node::{DisplayRow, Entry, entry_at_path, flatten};

pub struct Explorer {
    cwd: PathBuf,
    entries: Vec<Entry>,
    rows: Vec<DisplayRow>,
    selected: usize,
    scroll: usize,
    show_help: bool,
}

impl Explorer {
    pub fn new(cwd: PathBuf) -> Result<Self> {
        let entries = Entry::load_children(&cwd)?;
        let rows = Self::build_rows(&entries);
        Ok(Self { cwd, entries, rows, selected: 0, scroll: 0, show_help: false })
    }

    fn build_rows(entries: &[Entry]) -> Vec<DisplayRow> {
        let mut rows = vec![DisplayRow {
            prefix: String::new(),
            marker: "◁ ",
            name: "..".to_owned(),
            is_dir: true,
            tree_path: vec![],
        }];
        rows.extend(flatten(entries));
        rows
    }

    fn reload_cwd(&mut self) -> Result<()> {
        self.entries = Entry::load_children(&self.cwd)?;
        self.rows = Self::build_rows(&self.entries);
        self.selected = 0;
        self.scroll = 0;
        Ok(())
    }

    pub fn run(mut self) -> Result<()> {
        let mut out = stdout();
        terminal::enable_raw_mode()?;
        execute!(out, EnterAlternateScreen, Hide, Clear(ClearType::All))?;

        let result = self.event_loop(&mut out);

        execute!(out, Show, LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;
        result
    }

    fn event_loop(&mut self, out: &mut impl Write) -> Result<()> {
        loop {
            let size = terminal::size()?;
            let max_rows = size.1.saturating_sub(3) as usize;
            self.ensure_visible(max_rows);
            self.render(out, size)?;

            match event::read()? {
                Event::Key(k) if self.show_help => {
                    let _ = k;
                    self.show_help = false;
                }
                Event::Key(k) if k.code == KeyCode::Char('c')
                    && k.modifiers.contains(KeyModifiers::CONTROL) => return Ok(()),
                Event::Key(k) if k.code == KeyCode::Char('q') => return Ok(()),
                Event::Key(k) if k.code == KeyCode::Char('?') => self.show_help = true,
                Event::Key(k) if k.code == KeyCode::Up    => self.move_up(),
                Event::Key(k) if k.code == KeyCode::Down  => self.move_down(),
                Event::Key(k) if k.code == KeyCode::Left  => self.go_parent()?,
                Event::Key(k) if k.code == KeyCode::Right
                    || k.code == KeyCode::Enter => self.toggle_selected()?,
                _ => {}
            }
        }
    }

    fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    fn move_down(&mut self) {
        if self.selected + 1 < self.rows.len() {
            self.selected += 1;
        }
    }

    fn toggle_selected(&mut self) -> Result<()> {
        let Some(row) = self.rows.get(self.selected) else { return Ok(()) };

        if row.tree_path.is_empty() {
            return self.go_parent();
        }

        let tree_path = row.tree_path.clone();
        entry_at_path(&mut self.entries, &tree_path).toggle()?;
        self.rows = Self::build_rows(&self.entries);
        self.selected = self.selected.min(self.rows.len().saturating_sub(1));
        Ok(())
    }

    fn go_parent(&mut self) -> Result<()> {
        let Some(parent) = self.cwd.parent().map(|p| p.to_path_buf()) else { return Ok(()) };
        self.cwd = parent;
        self.reload_cwd()
    }

    fn ensure_visible(&mut self, max_rows: usize) {
        if max_rows == 0 { return; }
        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if self.selected >= self.scroll + max_rows {
            self.scroll = self.selected + 1 - max_rows;
        }
    }

    fn render(&self, out: &mut impl Write, (cols, terminal_rows): (u16, u16)) -> Result<()> {
        if self.show_help {
            return self.render_help(out, (cols, terminal_rows));
        }

        let max_rows = terminal_rows.saturating_sub(3) as usize;
        let start = self.scroll.min(self.rows.len());
        let end = (start + max_rows).min(self.rows.len());

        execute!(out, Clear(ClearType::All), MoveTo(0, 0))?;
        write!(out, "\rcwd: {}\r\n", self.cwd.display())?;

        for (i, row) in self.rows[start..end].iter().enumerate() {
            let is_selected = start + i == self.selected;
            if is_selected { execute!(out, SetAttribute(Attribute::Reverse))?; }
            write!(out, "\r{}{}", row.prefix, row.marker)?;
            let color = if row.is_dir { Color::Blue } else { Color::White };
            execute!(out, SetForegroundColor(color))?;
            write!(out, "{}", row.name)?;
            execute!(out, ResetColor)?;
            write!(out, "\r\n")?;
            if is_selected { execute!(out, SetAttribute(Attribute::Reset))?; }
        }

        // Separator
        execute!(out, MoveTo(0, terminal_rows.saturating_sub(2)))?;
        execute!(out, SetForegroundColor(Color::DarkGrey))?;
        write!(out, "\r{}\r\n", "─".repeat(cols as usize))?;
        execute!(out, ResetColor)?;

        // Footer
        execute!(out, MoveTo(0, terminal_rows.saturating_sub(1)), Clear(ClearType::CurrentLine))?;
        execute!(out, SetForegroundColor(Color::DarkGrey))?;
        write!(out, " Press ")?;
        execute!(out, SetForegroundColor(Color::Cyan))?;
        write!(out, "?")?;
        execute!(out, SetForegroundColor(Color::DarkGrey))?;
        write!(out, " for help")?;

        // Right-aligned count
        let count = format!("{}/{}", end.saturating_sub(start), self.rows.len());
        let count_col = (cols as usize).saturating_sub(count.len() + 1);
        execute!(out, MoveTo(count_col as u16, terminal_rows.saturating_sub(1)))?;
        execute!(out, SetForegroundColor(Color::Yellow))?;
        write!(out, "{count}")?;

        execute!(out, ResetColor)?;

        out.flush()?;
        Ok(())
    }

    fn render_help(&self, out: &mut impl Write, (cols, terminal_rows): (u16, u16)) -> Result<()> {
        execute!(out, Clear(ClearType::All), MoveTo(0, 0))?;

        let help = [
            ("↑ / ↓",       "Navigate up/down"),
            ("→ / Enter",   "Expand/collapse directory"),
            ("←",           "Go to parent directory"),
            ("q / Ctrl+C",  "Quit"),
            ("?",           "Toggle this help"),
        ];

        let box_width = 42;
        let pad_left = ((cols as usize).saturating_sub(box_width)) / 2;
        let pad_top = ((terminal_rows as usize).saturating_sub(help.len() + 4)) / 2;
        let indent = " ".repeat(pad_left);

        for _ in 0..pad_top {
            write!(out, "\r\n")?;
        }

        execute!(out, SetForegroundColor(Color::DarkGrey))?;
        write!(out, "\r{}╭{}╮\r\n", indent, "─".repeat(box_width - 2))?;

        execute!(out, SetForegroundColor(Color::Cyan))?;
        write!(out, "\r{}│{:^width$}│\r\n", indent, "Keybindings", width = box_width - 2)?;

        execute!(out, SetForegroundColor(Color::DarkGrey))?;
        write!(out, "\r{}├{}┤\r\n", indent, "─".repeat(box_width - 2))?;

        for (key, desc) in &help {
            write!(out, "\r{}│", indent)?;
            execute!(out, SetForegroundColor(Color::Cyan))?;
            write!(out, " {:<14}", key)?;
            execute!(out, SetForegroundColor(Color::White))?;
            write!(out, "{:<width$}", desc, width = box_width - 17)?;
            execute!(out, SetForegroundColor(Color::DarkGrey))?;
            write!(out, "│\r\n")?;
        }

        write!(out, "\r{}╰{}╯\r\n", indent, "─".repeat(box_width - 2))?;
        execute!(out, ResetColor)?;

        execute!(out, MoveTo(0, terminal_rows.saturating_sub(1)), Clear(ClearType::CurrentLine))?;
        execute!(out, SetForegroundColor(Color::DarkGrey))?;
        write!(out, " Press any key to close")?;
        execute!(out, ResetColor)?;

        out.flush()?;
        Ok(())
    }
}
