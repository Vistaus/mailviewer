/* mailservice.rs
 *
 * Copyright 2024 Alexandre Del Bigio
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */
use crate::config::VERSION;
use crate::message::attachment::Attachment;
use crate::message::message::{Message, MessageParser};
use std::cell::RefCell;
use std::path::Path;

pub struct MailService {
  parser: RefCell<Option<MessageParser>>,
  full_path: RefCell<Option<String>>,
  show_file_name: RefCell<bool>,
  signal_title_changed: RefCell<Option<Box<dyn Fn(&Self, &str) + 'static>>>,
}

impl MailService {
  pub fn new() -> Self {
    Self {
      parser: RefCell::new(None),
      full_path: RefCell::new(None),
      show_file_name: RefCell::new(true),
      signal_title_changed: RefCell::new(None),
    }
  }

  pub fn open_message(&self, fullpath: &str) -> Result<(), Box<dyn std::error::Error>> {
    if Path::new(fullpath).exists() == false {
      return Err(format!("File not found : {}", fullpath).into());
    }
    self.full_path.borrow_mut().replace(fullpath.to_string());
    let mut parser = MessageParser::new(fullpath);
    parser.parse()?;
    self.parser.borrow_mut().replace(parser);
    self.update_title();
    Ok(())
  }

  pub fn from(&self) -> String {
    if let Some(parser) = self.parser.borrow().as_ref() {
      return parser.from();
    }
    String::new()
  }

  pub fn to(&self) -> String {
    if let Some(parser) = self.parser.borrow().as_ref() {
      return parser.to();
    }
    String::new()
  }

  pub fn subject(&self) -> String {
    if let Some(parser) = self.parser.borrow().as_ref() {
      return parser.subject();
    }
    String::new()
  }

  pub fn date(&self) -> String {
    if let Some(parser) = self.parser.borrow().as_ref() {
      return parser.date();
    }
    String::new()
  }

  pub fn body_text(&self) -> Option<String> {
    if let Some(parser) = self.parser.borrow().as_ref() {
      return parser.body_text();
    }
    None
  }

  pub fn body_html(&self) -> Option<String> {
    if let Some(parser) = self.parser.borrow().as_ref() {
      return parser.body_html();
    }
    None
  }

  pub fn attachments(&self) -> Vec<Attachment> {
    if let Some(parser) = self.parser.borrow().as_ref() {
      return parser.attachments().clone();
    }
    vec![]
  }

  pub fn set_show_file_name(&self, show_file_name: bool) {
    log::debug!("set_show_file_name({})", show_file_name);
    self.show_file_name.replace(show_file_name);
    self.update_title();
  }

  pub fn get_fullpath(&self) -> Option<String> {
    self.full_path.borrow().clone()
  }

  pub fn connect_title_changed<F: Fn(&Self, &str) + 'static>(&self, f: F) {
    self.signal_title_changed.borrow_mut().replace(Box::new(f));
  }

  fn update_title(&self) {
    if let Some(callback) = self.signal_title_changed.borrow().as_ref() {
      if let Some(fullpath) = self.full_path.borrow().as_ref() {
        let title = self.get_title(fullpath);
        callback(self, &title);
      }
    }
  }

  fn get_title(&self, fullpath: &str) -> String {
    if *self.show_file_name.borrow() {
      if let Some(filename) = Path::new(fullpath).file_name() {
        return filename.to_string_lossy().to_string();
      }
    }
    format!("Mail Viewer v{}", VERSION)
  }
}

impl std::fmt::Debug for MailService {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("MailService")
      .field("fullpath", &self.full_path)
      .field("show_file_name", &self.show_file_name)
      .finish()
  }
}

#[cfg(test)]
mod tests {
  use crate::mailservice::MailService;
  use std::rc::Rc;

  #[test]
  fn new_mail_service() {
    let service = MailService::new();

    assert!(service.parser.borrow().is_none());
    assert!(service.full_path.borrow().is_none());
    assert_eq!(*service.show_file_name.borrow(), true);
  }

  #[test]
  fn open_mail_success() {
    let mail_service = MailService::new();
    let service = mail_service;
    let fullpath = "sample.eml";

    assert!(service.open_message(fullpath).is_ok());
    assert_eq!(service.get_fullpath().unwrap(), fullpath.to_string());
    assert_eq!(service.from(), "John Doe <john@moon.space>");
    assert_eq!(service.to(), "Lucas <lucas@mercure.space>");
    assert_eq!(service.subject(), "Lorem ipsum");
    assert_eq!(service.date(), "2024-10-23 12:27:21");
  }

  #[test]
  fn open_mail_file_not_found() {
    let service = MailService::new();
    let result = service.open_message("path/to/nonexistent.eml");

    assert!(result.is_err());
    assert_eq!(
      format!("{}", result.unwrap_err()),
      "File not found : path/to/nonexistent.eml"
    );
  }

  #[test]
  fn get_text() {
    let service = MailService::new();
    service.open_message("sample.eml").unwrap();
    let text = service.body_text().unwrap();

    assert!(text.contains("Lorem ipsum dolor sit amet, consectetur adipiscing elit"));
  }

  #[test]
  fn get_html() {
    let service = MailService::new();
    service.open_message("sample.eml").unwrap();
    let html = service.body_html().unwrap();

    assert!(html.contains("Hello Lucas,"));
  }

  #[test]
  fn get_attachments() {
    let service = MailService::new();

    service.open_message("sample.eml").unwrap();
    let attachments = service.attachments();

    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].filename, "Deus_Gnome.png");
  }

  #[test]
  fn update_title_with_show_file_name() {
    let service = MailService::new();
    service.open_message("sample.eml").unwrap();
    service.set_show_file_name(true);
    assert_eq!(service.get_title("sample.eml"), "sample.eml");
  }

  #[test]
  fn update_title_without_show_file_name() {
    let service = MailService::new();
    service.set_show_file_name(false);
    assert_eq!(
      service.get_title("sample.eml"),
      format!("Mail Viewer v{}", crate::config::VERSION)
    );
  }

  #[test]
  fn connect_title_changed() {
    let service = MailService::new();
    let title_changed_called = Rc::new(std::cell::RefCell::new(false));
    let title_changed_called_clone = Rc::clone(&title_changed_called);
    service.connect_title_changed(move |_, _| {
      *title_changed_called_clone.borrow_mut() = true;
    });
    service.open_message("sample.eml").unwrap();
    service.set_show_file_name(false);
    assert!(*title_changed_called.borrow());
  }
}
