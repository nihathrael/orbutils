#![deny(warnings)]

extern crate orbtk;

use orbtk::{Action, Menu, Point, Rect, TextBox, Window, Label, Button};
use orbtk::callback::Click;
use orbtk::place::Place;

use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::Arc;

fn main() {
    let path_option = env::args().nth(1);

    let title = if let Some(ref path) = path_option {
        format!("{} - Editor", path)
    } else {
        format!("Editor")
    };

    let mut window = Window::new(Rect::new(100, 100, 576, 420), &title);

    let text_box = TextBox::new()
        .position(0, 16)
        .size(576, 404)
        .place(&mut window);

    if let Some(ref path) = path_option {
        match File::open(path) {
            Ok(mut file) => {
                let mut text = String::new();
                match file.read_to_string(&mut text) {
                    Ok(_) => text_box.text.set(text),
                    Err(err) => println!("Failed to read {}: {}", path, err),
                }
            }
            Err(err) => println!("Failed to open {}: {}", path, err),
        }
    }

    let mut menu = Menu::new("File").position(0, 0).size(32, 16);

    menu.add_action(Action::new("Open").on_click(|_action: &Action, _point: Point| {
        println!("Open");
    }));

    menu.add_separator();

    let textbox_save_clone = text_box.clone();
    menu.add_action(Action::new("Save").on_click(move |_action: &Action, _point: Point| {
        println!("Save");
        if let Some(ref path) = path_option {
            let text = textbox_save_clone.text.get();
            save_text_to_file(text, path);
        } else {
            println!("Need to create file!");
        }
    }));

    let textbox_save_as_clone = text_box.clone();
    menu.add_action(Action::new("Save As").on_click(move |_action: &Action, _point: Point| {
        save_as(textbox_save_as_clone.clone());
    }));

    menu.add_separator();

    menu.add_action(Action::new("Close").on_click(|_action: &Action, _point: Point| {
        println!("Close");
    }));

    menu.place(&mut window);

    window.exec();
}

fn save_text_to_file(text: String, path: &String) {
    match File::create(path) {
        Ok(mut file) => {
            match file.write(&mut text.as_bytes()) {
                Ok(_) => {
                    match file.set_len(text.len() as u64) {
                        Ok(_) => println!("Successfully saved {}", path),
                        Err(err) => println!("Failed to truncate {}: {}", path, err),
                    }
                }
                Err(err) => println!("Failed to write {}: {}", path, err),
            }
        }
        Err(err) => println!("Failed to open {}: {}", path, err),
    }
}

fn save_as(text_box: Arc<TextBox>) {
    println!("Save as new!");
    let mut window = Window::new(Rect::new(100, 100, 200, 100), "Save as...");

    Label::new()
        .position(0, 0)
        .size(200, 16)
        .text("Enter a filename:")
        .place(&mut window);


    let filename_text_box = TextBox::new()
        .position(0, 16)
        .size(200, 16)
        .text("file.txt")
        .place(&mut window);

    Button::new()
        .position(0, 32)
        .size(48, filename_text_box.rect.get().height)
        .text("Save")
        .on_click(move |_button: &Button, _point: Point| {
            save_text_to_file(text_box.text.get(), &filename_text_box.text.get());
            println!("Saving as: {}", filename_text_box.text.get());
        })
        .place(&mut window);

    window.exec();
}
