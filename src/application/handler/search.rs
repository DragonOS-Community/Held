use crate::application::mode::ModeData;
use crate::application::Application;
use crate::errors::*;
use crate::util::{position::Position, range::Range};
use crossterm::event::KeyCode;

pub fn exec_search(app: &mut Application) -> Result<()> {
    if let ModeData::Search(ref mut search_data) = app.mode {
        search_data.is_exec_search = true;
        if let Some(ref mut buffer) = app.workspace.current_buffer {
            let search_string = search_data.search_string.clone();
            let search_result = buffer.search(&search_string);

            let fixed_offset = search_string.len();
            let ranges: Vec<Range> = search_result
                .into_iter()
                .map(|pos| {
                    let end_position = Position {
                        line: pos.line,
                        offset: pos.offset + fixed_offset,
                    };
                    Range::new(pos, end_position)
                })
                .collect();

            search_data.search_result = ranges;
        }
    }
    Ok(())
}

pub fn input_search_data(app: &mut Application) -> Result<()> {
    if let Some(key) = app.monitor.last_key {
        if let KeyCode::Char(c) = key.code {
            if let ModeData::Search(ref mut search_data) = app.mode {
                search_data
                    .search_string
                    .insert(search_data.search_string.len(), c);
            }
        }
    }
    Ok(())
}

pub fn backspace(app: &mut Application) -> Result<()> {
    if let ModeData::Search(ref mut search_data) = app.mode {
        if search_data.search_string.len() > 0 {
            search_data
                .search_string
                .remove(search_data.search_string.len() - 1);
        }
    }
    Ok(())
}

pub fn last_result(app: &mut Application) -> Result<()> {
    if let ModeData::Search(ref mut search_data) = app.mode {
        if search_data.is_exec_search == true {
            search_data.search_result_index =
                (search_data.search_result_index + search_data.search_result.len() - 1)
                    % (search_data.search_result.len() - 1);
        }
    }
    Ok(())
}

pub fn next_result(app: &mut Application) -> Result<()> {
    if let ModeData::Search(ref mut search_data) = app.mode {
        if search_data.is_exec_search == true {
            search_data.search_result_index =
                (search_data.search_result_index + 1) % (search_data.search_result.len() - 1);
        }
    }
    Ok(())
}

pub fn clear(app: &mut Application) -> Result<()> {
    if let ModeData::Search(ref mut search_data) = app.mode {
        search_data.clear();
    }
    Ok(())
}
