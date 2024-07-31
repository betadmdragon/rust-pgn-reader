use rust_pgn_extract::config::Config;


pub struct FilteringVisitor<'a> {
    config: &'a Config,
    should_write: bool, // Flag to indicate whether the current game should be written to the output
}

impl<'a> FilteringVisitor<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self {
            config,
            should_write: true,
        }
    }
}

impl<'a> Visitor for FilteringVisitor<'a> {
    type Result = ();

    fn header(&mut self, _dummy: &[u8], tag: RawHeader) {
        // Access the tag name and value from the RawHeader
        let tag_name = tag.0;
        let tag_value = tag.1;
        match tag_name {
            b"TimeControl" => {
                if let Ok(time_control) = std::str::from_utf8(tag_value).unwrap_or("").parse::<u16>() {
                    if time_control < self.config.filters.time_control.min {
                        self.should_write = false;
                    }
                } else {
                    // Handle invalid TimeControl values (e.g., not a number)
                    eprintln!("Invalid TimeControl value: {:?}", tag_value);
                    self.should_write = false;
                }
            }
            b"WhiteElo" | b"BlackElo" => {
                if let Ok(elo) = std::str::from_utf8(tag_value).unwrap_or("").parse::<u16>() {
                    let min_elo = if tag_name == b"WhiteElo" {
                        self.config.filters.white_elo.min
                    } else {
                        self.config.filters.black_elo.min
                    };
                    if elo < min_elo {
                        self.should_write = false;
                    }
                } else {
                    // Handle invalid ELO values (e.g., not a number)
                    eprintln!("Invalid ELO value: {:?}", tag_value);
                    self.should_write = false;
                }
            }
            b"White" | b"Black" => {
                if self.config.filters.exclude_bots
                    && std::str::from_utf8(tag_value).unwrap_or("").to_lowercase().contains("bot")
                {
                    self.should_write = false;
                }
            }
            b"WhiteTitle" | b"BlackTitle" => {
                if self.config.filters.exclude_bots && tag_value == b"BOT" {
                    self.should_write = false;
                }
            }
            _ => {} // Ignore other headers
        }
    }

    // `end_headers` is called after all the headers for a game have been processed. 
    fn end_headers(&mut self) -> Skip {
        // If the `should_write` flag is false at this point, it means
        // that one of the header fields failed the filter criteria.
        // In that case, we skip the rest of the game.
        if !self.should_write {
            return Skip(false);
        }
        Skip(true)
    }

    fn end_game(&mut self) -> Self::Result {
        // This method is called at the end of each game
        // We don't need to do anything here for our filtering logic
    }
}
