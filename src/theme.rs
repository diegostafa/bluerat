use std::str::FromStr;

use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, BorderType, Borders, TableState};
use ratatui_helpers::stateful_table::{IndexedRow, StatefulTable, TableStyle, Tabular};

use crate::globals::CONFIG;

pub struct StyledWidget;
impl StyledWidget {
    pub fn table<'a, T: Tabular>(
        data: Vec<T>,
        state: TableState,
        title: Option<String>,
    ) -> StatefulTable<'a, T> {
        StatefulTable::new(data, state, Self::table_style(), title)
    }
    pub fn indexed_table<'a, T: Tabular>(
        data: Vec<T>,
        state: TableState,
        title: Option<String>,
    ) -> StatefulTable<'a, IndexedRow<T>> {
        StatefulTable::new(IndexedRow::from(data), state, Self::table_style(), title)
    }
    pub fn block<'a>() -> Block<'a> {
        let mut block = Block::new();
        if CONFIG.theme.borders {
            block = block.borders(Borders::ALL).border_style(
                Style::default().fg(Color::from_str(&CONFIG.theme.border_color).unwrap()),
            )
        }

        if CONFIG.theme.rounded_borders {
            block = block.border_type(BorderType::Rounded)
        }
        block
    }

    fn table_style<'a>() -> TableStyle<'a> {
        TableStyle {
            table: Style::default(),
            header: Style::default()
                .fg(Color::from_str(&CONFIG.theme.fg_header_color).unwrap())
                .bg(Color::from_str(&CONFIG.theme.bg_header_color).unwrap()),
            block: Some(Self::block()),
            highlight: Style::default()
                .fg(Color::from_str(&CONFIG.theme.fg_selected_color).unwrap())
                .bg(Color::from_str(&CONFIG.theme.bg_selected_color).unwrap()),
            column_spacing: CONFIG.theme.column_spacing,
        }
    }
}
