# sd-journal-rs

Another Rust binding for `sd-journal(3)`.

## Work In Progress

- [x] Open the system journal for reading
  - [x] impl `sd_journal_[open|close]`
  - [ ] impl `sd_journal_open_directory[_fd]`
  - [ ] impl `sd_journal_open_files[_fd]`
- [x] Journal disk usage `sd_journal_get_usage`
- [x] Seek to a position in the journal
  - [x] impl `sd_journal_seek_[head|tail]`
  - [x] impl `sd_journal_seek_[monotonic|realtime]_usec`
  - [ ] impl `sd_journal_seek_cursor`
- [x] Advance or set back the read pointer in the journal
  - [x] impl `sd_journal_[next|previous][_skip]`
  - [ ] impl `SD_JOURNAL_FOREACH[_BACKWARDS]`
- [x] Read data fields from the current journal entry
  - [x] impl `sd_journal_get_data`
  - [x] impl `sd_journal_[enumerate|restart]_data`, `SD_JOURNAL_FOREACH_DATA`
  - [ ] impl `sd_journal_[get|set]_data_threshold`
- [ ] Retrieve message catalog entry
  - [ ] impl `sd_journal_get_catalog[_for_message_id]`
- [ ] Get/test cursor string for the current journal entry
  - [ ] impl `sd_journal_[get|test]_cursor`
- [ ] Read used field names from the journal
  - [ ] impl `sd_journal_[enumerate|restart]_fields`, `SD_JOURNAL_FOREACH_FIELD`
- [ ] Add or remove entry matches
  - [ ] impl `sd_journal_add_match`
  - [ ] and more ...
- [ ] Read unique data fields from the journal
  - [ ] impl `sd_journal_query_unique`
  - [ ] and more ...
- [ ] Journal change notification interface
  - [ ] impl `sd_journal_get_[fd|events|timeout]`
  - [ ] and more ...
- [ ] Submit log entries to the journal
  - [ ] impl `sd_journal_[print|send]`
  - [ ] and more ...
