use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, Entry, Label, ListBox, Orientation,
    ScrolledWindow, CheckButton, Paned, FileChooserDialog, FileChooserAction, FileFilter,
    ResponseType, Image,
};
use gtk4::gio;
use gtk4::glib::clone::Downgrade;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::env;

use crate::desktop_file::{scan_desktop_files, DesktopEntry};

#[derive(Clone)]
struct MimeChoice {
    extension: String,
    mime_type: String,
    description: String,
}

pub struct MainWindow {
    window: ApplicationWindow,
}

impl MainWindow {
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Manchatz Desktop Entry Manager")
            .default_width(1000)
            .default_height(600)
            .build();

        let main_box = GtkBox::new(Orientation::Vertical, 0);
        window.set_child(Some(&main_box));

        let paned = Paned::new(Orientation::Horizontal);
        paned.set_position(450); // Set initial position to make left pane wider
        main_box.append(&paned);

        // Left side: list of desktop files
        let left_box = GtkBox::new(Orientation::Vertical, 5);
        left_box.set_margin_start(5);
        left_box.set_margin_end(5);
        left_box.set_margin_top(5);
        left_box.set_margin_bottom(5);
        left_box.set_width_request(400); // Set minimum width for left pane

        let search_entry = Entry::builder()
            .placeholder_text("Search applications...")
            .build();
        left_box.append(&search_entry);

        // New Entry button
        let new_entry_button = Button::with_label("+ New Entry");
        new_entry_button.set_margin_top(5);
        new_entry_button.set_margin_bottom(5);
        left_box.append(&new_entry_button);

        let scrolled = ScrolledWindow::builder()
            .hexpand(true)
            .vexpand(true)
            .build();

        let list_box = ListBox::new();
        list_box.set_selection_mode(gtk4::SelectionMode::Single);
        scrolled.set_child(Some(&list_box));
        left_box.append(&scrolled);

        paned.set_start_child(Some(&left_box));
        paned.set_resize_start_child(true);
        paned.set_shrink_start_child(false);

        // Right side: details/editor
        let right_box = GtkBox::new(Orientation::Vertical, 10);
        right_box.set_margin_start(10);
        right_box.set_margin_end(10);
        right_box.set_margin_top(10);
        right_box.set_margin_bottom(10);

        let details_label = Label::new(Some("Select an application to view/edit"));
        details_label.set_halign(gtk4::Align::Start);
        right_box.append(&details_label);

        // Entry fields container
        let editor_box = GtkBox::new(Orientation::Vertical, 10);
        editor_box.set_visible(false);

        // Name field
        let name_box = GtkBox::new(Orientation::Vertical, 5);
        let name_label = Label::new(Some("Name:"));
        name_label.set_halign(gtk4::Align::Start);
        let name_entry = Entry::new();
        name_box.append(&name_label);
        name_box.append(&name_entry);
        editor_box.append(&name_box);

        // Exec field
        let exec_box = GtkBox::new(Orientation::Vertical, 5);
        let exec_label = Label::new(Some("Command:"));
        exec_label.set_halign(gtk4::Align::Start);
        let exec_input_box = GtkBox::new(Orientation::Horizontal, 5);
        let exec_entry = Entry::new();
        exec_entry.set_hexpand(true);
        let exec_browse_button = Button::with_label("Browse...");
        exec_input_box.append(&exec_entry);
        exec_input_box.append(&exec_browse_button);
        exec_box.append(&exec_label);
        exec_box.append(&exec_input_box);
        editor_box.append(&exec_box);

        // Icon field with preview
        let icon_box = GtkBox::new(Orientation::Vertical, 5);
        let icon_label = Label::new(Some("Icon:"));
        icon_label.set_halign(gtk4::Align::Start);

        let icon_main_box = GtkBox::new(Orientation::Horizontal, 10);

        // Icon preview
        let icon_preview = Image::from_icon_name("application-x-executable");
        icon_preview.set_pixel_size(64);
        icon_main_box.append(&icon_preview);

        // Icon input box
        let icon_input_container = GtkBox::new(Orientation::Vertical, 5);
        icon_input_container.set_hexpand(true);
        let icon_input_box = GtkBox::new(Orientation::Horizontal, 5);
        let icon_entry = Entry::new();
        icon_entry.set_hexpand(true);
        let icon_browse_button = Button::with_label("Browse...");
        icon_input_box.append(&icon_entry);
        icon_input_box.append(&icon_browse_button);
        icon_input_container.append(&icon_input_box);

        icon_main_box.append(&icon_input_container);

        icon_box.append(&icon_label);
        icon_box.append(&icon_main_box);
        editor_box.append(&icon_box);

        // Comment field
        let comment_box = GtkBox::new(Orientation::Vertical, 5);
        let comment_label = Label::new(Some("Comment:"));
        comment_label.set_halign(gtk4::Align::Start);
        let comment_entry = Entry::new();
        comment_box.append(&comment_label);
        comment_box.append(&comment_entry);
        editor_box.append(&comment_box);

        // Categories field
        let categories_box = GtkBox::new(Orientation::Vertical, 5);
        let categories_label = Label::new(Some("Categories:"));
        categories_label.set_halign(gtk4::Align::Start);
        let categories_entry = Entry::new();
        categories_box.append(&categories_label);
        categories_box.append(&categories_entry);
        editor_box.append(&categories_box);

        // File association management
        let mime_box = GtkBox::new(Orientation::Vertical, 5);
        let mime_label = Label::new(Some("Associated file types:"));
        mime_label.set_halign(gtk4::Align::Start);

        let mime_scrolled = ScrolledWindow::builder()
            .min_content_height(60)
            .max_content_height(140)
            .hexpand(true)
            .vexpand(false)
            .build();

        let mime_list = ListBox::new();
        mime_list.set_selection_mode(gtk4::SelectionMode::None);
        mime_scrolled.set_child(Some(&mime_list));

        let mime_buttons_box = GtkBox::new(Orientation::Horizontal, 5);
        let add_mime_button = Button::with_label("Add file association...");
        add_mime_button.set_sensitive(false);
        mime_buttons_box.append(&add_mime_button);

        mime_box.append(&mime_label);
        mime_box.append(&mime_scrolled);
        mime_box.append(&mime_buttons_box);
        editor_box.append(&mime_box);

        // Terminal checkbox
        let terminal_check = CheckButton::with_label("Run in terminal");
        editor_box.append(&terminal_check);

        // Path display with read-only indicator
        let path_box = GtkBox::new(Orientation::Vertical, 5);
        let path_label = Label::new(Some("File path:"));
        path_label.set_halign(gtk4::Align::Start);

        let path_display_box = GtkBox::new(Orientation::Horizontal, 8);
        let path_display = Label::new(None);
        path_display.set_halign(gtk4::Align::Start);
        path_display.set_selectable(true);
        path_display.set_wrap(true);
        path_display.set_hexpand(true);

        let readonly_icon = Image::from_icon_name("changes-prevent-symbolic");
        readonly_icon.set_pixel_size(16);
        readonly_icon.set_opacity(0.6);
        readonly_icon.set_visible(false); // Hidden by default
        readonly_icon.set_tooltip_text(Some("Read-only: elevated privileges required"));

        path_display_box.append(&path_display);
        path_display_box.append(&readonly_icon);

        path_box.append(&path_label);
        path_box.append(&path_display_box);
        editor_box.append(&path_box);

        // Buttons
        let button_box = GtkBox::new(Orientation::Horizontal, 10);
        button_box.set_margin_top(20);
        let save_button = Button::with_label("Save Changes");
        let delete_button = Button::with_label("Delete Entry");
        delete_button.add_css_class("destructive-action");
        let refresh_button = Button::with_label("Refresh List");
        button_box.append(&save_button);
        button_box.append(&delete_button);
        button_box.append(&refresh_button);
        editor_box.append(&button_box);

        right_box.append(&editor_box);
        paned.set_end_child(Some(&right_box));

        // Store current selection
        let current_entry: Rc<RefCell<Option<DesktopEntry>>> = Rc::new(RefCell::new(None));
        let current_row_widget: Rc<RefCell<Option<gtk4::Widget>>> = Rc::new(RefCell::new(None));
        let mime_types_state: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let mime_extensions_state: Rc<RefCell<HashMap<String, String>>> =
            Rc::new(RefCell::new(HashMap::new()));
        let known_mime_choices = Rc::new(load_known_extensions());
        let known_mime_map: Rc<RefCell<HashMap<String, String>>> =
            Rc::new(RefCell::new(build_mime_extension_map(&known_mime_choices)));

        // Load desktop files
        let entries = scan_desktop_files();
        let all_entries = Rc::new(RefCell::new(entries.clone()));

        // Map to track which entry corresponds to each row widget
        let row_entry_map: Rc<RefCell<HashMap<gtk4::Widget, DesktopEntry>>> = Rc::new(RefCell::new(HashMap::new()));

        // Populate list
        for entry in &entries {
            let row = create_list_row(&entry.name, &entry.comment, &entry.icon);
            let widget = row.clone().upcast::<gtk4::Widget>();
            row_entry_map.borrow_mut().insert(widget, entry.clone());
            list_box.append(&row);
        }

        // Handle selection - We'll add permission check fields later
        let editor_box_clone = editor_box.clone();
        let details_label_clone = details_label.clone();
        let name_entry_clone = name_entry.clone();
        let exec_entry_clone = exec_entry.clone();
        let icon_entry_clone = icon_entry.clone();
        let icon_preview_clone = icon_preview.clone();
        let comment_entry_clone = comment_entry.clone();
        let categories_entry_clone = categories_entry.clone();
        let terminal_check_clone = terminal_check.clone();
        let path_display_clone = path_display.clone();
        let current_entry_clone = current_entry.clone();
        let current_row_widget_clone = current_row_widget.clone();
        let row_entry_map_clone = row_entry_map.clone();
        let name_entry_perm = name_entry.clone();
        let exec_entry_perm = exec_entry.clone();
        let icon_entry_perm = icon_entry.clone();
        let comment_entry_perm = comment_entry.clone();
        let categories_entry_perm = categories_entry.clone();
        let terminal_check_perm = terminal_check.clone();
        let save_button_perm = save_button.clone();
        let delete_button_perm = delete_button.clone();
        let exec_browse_button_perm = exec_browse_button.clone();
        let icon_browse_button_perm = icon_browse_button.clone();
        let readonly_icon_clone = readonly_icon.clone();
        let mime_list_clone = mime_list.clone();
        let mime_types_state_clone = mime_types_state.clone();
        let mime_extensions_state_clone = mime_extensions_state.clone();
        let add_mime_button_perm = add_mime_button.clone();
        let known_mime_map_clone = known_mime_map.clone();

        list_box.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                let widget = row.child().unwrap();
                let map = row_entry_map_clone.borrow();
                if let Some(entry) = map.get(&widget) {
                    details_label_clone.set_visible(false);
                    editor_box_clone.set_visible(true);

                    name_entry_clone.set_text(&entry.name);
                    exec_entry_clone.set_text(&entry.exec);
                    icon_entry_clone.set_text(&entry.icon);
                    comment_entry_clone.set_text(&entry.comment);
                    categories_entry_clone.set_text(&entry.categories);
                    terminal_check_clone.set_active(entry.terminal);
                    path_display_clone.set_text(&entry.path.display().to_string());

                    // Update icon preview
                    update_icon_preview(&icon_preview_clone, &entry.icon);

                    {
                        let mut state = mime_types_state_clone.borrow_mut();
                        state.clear();
                        state.extend(entry.mime_types.iter().cloned());
                    }
                    {
                        let mut ext_state = mime_extensions_state_clone.borrow_mut();
                        ext_state.clear();
                        for (mime, ext) in &entry.mime_extensions {
                            ext_state.insert(mime.clone(), ext.clone());
                        }
                    }
                    clear_list_box(&mime_list_clone);
                    let existing_mimes: Vec<String> = mime_types_state_clone.borrow().clone();
                    for mime in existing_mimes {
                        let extension_owned = if let Some(ext) =
                            mime_extensions_state_clone.borrow().get(&mime).cloned()
                        {
                            Some(ext)
                        } else {
                            known_mime_map_clone.borrow().get(&mime).cloned()
                        };
                        append_mime_row(
                            &mime_list_clone,
                            &mime,
                            extension_owned.as_deref(),
                            mime_types_state_clone.clone(),
                            mime_extensions_state_clone.clone(),
                        );
                    }

                    *current_entry_clone.borrow_mut() = Some(entry.clone());
                    *current_row_widget_clone.borrow_mut() = Some(widget.clone());

                    // Check permissions and enable/disable controls
                    let can_write = can_write_file(&entry.path);
                    name_entry_perm.set_sensitive(can_write);
                    exec_entry_perm.set_sensitive(can_write);
                    icon_entry_perm.set_sensitive(can_write);
                    comment_entry_perm.set_sensitive(can_write);
                    categories_entry_perm.set_sensitive(can_write);
                    terminal_check_perm.set_sensitive(can_write);
                    save_button_perm.set_sensitive(can_write);
                    delete_button_perm.set_sensitive(can_write);
                    exec_browse_button_perm.set_sensitive(can_write);
                    icon_browse_button_perm.set_sensitive(can_write);
                    add_mime_button_perm.set_sensitive(can_write);

                    // Show/hide readonly icon
                    readonly_icon_clone.set_visible(!can_write);

                    if !can_write {
                        println!("Note: {} is read-only. You need elevated privileges to edit system-wide applications.", entry.path.display());
                    }
                }
            }
        });

        // Save button handler
        let current_entry_clone = current_entry.clone();
        let current_row_widget_clone = current_row_widget.clone();
        let name_entry_clone = name_entry.clone();
        let exec_entry_clone = exec_entry.clone();
        let icon_entry_clone = icon_entry.clone();
        let comment_entry_clone = comment_entry.clone();
        let categories_entry_clone = categories_entry.clone();
        let terminal_check_clone = terminal_check.clone();
        let mime_types_state_clone = mime_types_state.clone();
        let mime_extensions_state_clone = mime_extensions_state.clone();

        save_button.connect_clicked(move |_| {
            if let Some(ref mut entry) = *current_entry_clone.borrow_mut() {
                entry.name = name_entry_clone.text().to_string();
                entry.exec = exec_entry_clone.text().to_string();
                entry.icon = icon_entry_clone.text().to_string();
                entry.comment = comment_entry_clone.text().to_string();
                entry.categories = categories_entry_clone.text().to_string();
                entry.terminal = terminal_check_clone.is_active();
                entry.mime_types = mime_types_state_clone.borrow().clone();
                entry.mime_extensions = mime_extensions_state_clone.borrow().clone();

                match entry.save() {
                    Ok(_) => {
                        println!("Changes saved successfully!");
                        // Update the icon in the list after saving
                        update_row_icon(&current_row_widget_clone, &entry.icon);
                    }
                    Err(e) => {
                        eprintln!("Error saving file: {}", e);
                    }
                }
            }
        });

        // Refresh button handler
        let list_box_clone = list_box.clone();
        let all_entries_clone = all_entries.clone();
        let row_entry_map_clone = row_entry_map.clone();
        refresh_button.connect_clicked(move |_| {
            while let Some(child) = list_box_clone.first_child() {
                list_box_clone.remove(&child);
            }

            row_entry_map_clone.borrow_mut().clear();

            let entries = scan_desktop_files();
            *all_entries_clone.borrow_mut() = entries.clone();

            for entry in &entries {
                let row = create_list_row(&entry.name, &entry.comment, &entry.icon);
                let widget = row.clone().upcast::<gtk4::Widget>();
                row_entry_map_clone.borrow_mut().insert(widget, entry.clone());
                list_box_clone.append(&row);
            }
        });

        // Search functionality - optimized to hide/show instead of recreating widgets
        let list_box_clone = list_box.clone();
        let row_entry_map_clone = row_entry_map.clone();
        search_entry.connect_changed(move |entry| {
            let search_text = entry.text().to_lowercase();
            let map = row_entry_map_clone.borrow();

            // Instead of recreating widgets, just hide/show existing rows
            let mut child = list_box_clone.first_child();
            while let Some(row) = child {
                if let Some(list_row) = row.downcast_ref::<gtk4::ListBoxRow>() {
                    if let Some(row_child) = list_row.child() {
                        if let Some(desktop_entry) = map.get(&row_child) {
                            let matches = search_text.is_empty()
                                || desktop_entry.name.to_lowercase().contains(&search_text)
                                || desktop_entry.comment.to_lowercase().contains(&search_text);
                            list_row.set_visible(matches);
                        }
                    }
                }
                child = row.next_sibling();
            }
        });

        // New Entry button handler
        let editor_box_clone = editor_box.clone();
        let details_label_clone = details_label.clone();
        let name_entry_clone = name_entry.clone();
        let exec_entry_clone = exec_entry.clone();
        let icon_entry_clone = icon_entry.clone();
        let comment_entry_clone = comment_entry.clone();
        let categories_entry_clone = categories_entry.clone();
        let terminal_check_clone = terminal_check.clone();
        let path_display_clone = path_display.clone();
        let current_entry_clone = current_entry.clone();
        let list_box_clone = list_box.clone();
        let mime_list_clone = mime_list.clone();
        let mime_types_state_clone = mime_types_state.clone();
        let mime_extensions_state_clone = mime_extensions_state.clone();
        let add_mime_button_clone = add_mime_button.clone();

        new_entry_button.connect_clicked(move |_| {
            // Create a new desktop entry
            let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/tmp"));
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let filename = format!("new-application-{}.desktop", timestamp);
            let path = std::path::PathBuf::from(format!("{}/.local/share/applications/{}", home, filename));

            let new_entry = crate::desktop_file::DesktopEntry {
                path: path.clone(),
                name: String::from("New Application"),
                exec: String::from(""),
                icon: String::from("application-x-executable"),
                comment: String::from(""),
                terminal: false,
                categories: String::from(""),
                entry_type: String::from("Application"),
                mime_types: Vec::new(),
                mime_extensions: HashMap::new(),
            };

            // Clear list selection
            list_box_clone.unselect_all();

            // Show editor with new entry
            details_label_clone.set_visible(false);
            editor_box_clone.set_visible(true);

            name_entry_clone.set_text(&new_entry.name);
            exec_entry_clone.set_text(&new_entry.exec);
            icon_entry_clone.set_text(&new_entry.icon);
            comment_entry_clone.set_text(&new_entry.comment);
            categories_entry_clone.set_text(&new_entry.categories);
            terminal_check_clone.set_active(new_entry.terminal);
            path_display_clone.set_text(&new_entry.path.display().to_string());

            {
                mime_types_state_clone.borrow_mut().clear();
            }
            {
                mime_extensions_state_clone.borrow_mut().clear();
            }
            clear_list_box(&mime_list_clone);
            add_mime_button_clone.set_sensitive(true);

            *current_entry_clone.borrow_mut() = Some(new_entry);

            println!("New entry created. Fill in the details and click Save Changes.");
        });

        // File association button handler
        let window_clone = window.clone();
        let mime_list_clone = mime_list.clone();
        let mime_types_state_clone = mime_types_state.clone();
        let mime_extensions_state_clone = mime_extensions_state.clone();
        let current_entry_clone = current_entry.clone();
        let known_mime_choices_clone = known_mime_choices.clone();
        let known_mime_map_clone_2 = known_mime_map.clone();
        add_mime_button.connect_clicked(move |_| {
            show_mime_selection_dialog(
                &window_clone,
                known_mime_choices_clone.clone(),
                known_mime_map_clone_2.clone(),
                mime_types_state_clone.clone(),
                mime_extensions_state_clone.clone(),
                &mime_list_clone,
                current_entry_clone.clone(),
            );
        });

        // Command/Exec browse button handler
        let exec_entry_clone = exec_entry.clone();
        let window_clone = window.clone();
        exec_browse_button.connect_clicked(move |_| {
            let dialog = FileChooserDialog::new(
                Some("Select Executable"),
                Some(&window_clone),
                FileChooserAction::Open,
                &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)],
            );

            let exec_entry_clone2 = exec_entry_clone.clone();
            dialog.connect_response(move |dialog, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            exec_entry_clone2.set_text(&path.display().to_string());
                        }
                    }
                }
                dialog.close();
            });

            dialog.show();
        });

        // Icon browse button handler
        let icon_entry_clone = icon_entry.clone();
        let window_clone = window.clone();
        icon_browse_button.connect_clicked(move |_| {
            let dialog = FileChooserDialog::new(
                Some("Select Icon File"),
                Some(&window_clone),
                FileChooserAction::Open,
                &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Accept)],
            );

            // Add image file filter
            let filter = FileFilter::new();
            filter.set_name(Some("Image Files"));
            filter.add_mime_type("image/png");
            filter.add_mime_type("image/jpeg");
            filter.add_mime_type("image/svg+xml");
            filter.add_pattern("*.png");
            filter.add_pattern("*.jpg");
            filter.add_pattern("*.jpeg");
            filter.add_pattern("*.svg");
            dialog.add_filter(&filter);

            let icon_entry_clone2 = icon_entry_clone.clone();
            dialog.connect_response(move |dialog, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = dialog.file() {
                        if let Some(path) = file.path() {
                            let path_str = path.display().to_string();
                            icon_entry_clone2.set_text(&path_str);
                        }
                    }
                }
                dialog.close();
            });

            dialog.show();
        });

        // Icon entry change handler to update preview in real-time
        let icon_preview_clone = icon_preview.clone();
        icon_entry.connect_changed(move |entry| {
            let icon_name = entry.text().to_string();
            update_icon_preview(&icon_preview_clone, &icon_name);
        });

        // Delete button handler
        let current_entry_clone = current_entry.clone();
        let current_row_widget_clone = current_row_widget.clone();
        let list_box_clone = list_box.clone();
        let row_entry_map_clone = row_entry_map.clone();
        let all_entries_clone = all_entries.clone();
        let editor_box_clone = editor_box.clone();
        let details_label_clone = details_label.clone();

        delete_button.connect_clicked(move |_| {
            if let Some(entry) = &*current_entry_clone.borrow() {
                // Check if we have permission to delete this file
                if !can_write_file(&entry.path) {
                    eprintln!("Error: No permission to delete {}. System-wide applications require elevated privileges.", entry.path.display());
                    return;
                }

                // Try to delete the file
                match std::fs::remove_file(&entry.path) {
                    Ok(_) => {
                        println!("Successfully deleted: {}", entry.path.display());

                        // Remove from row_entry_map
                        if let Some(ref widget) = *current_row_widget_clone.borrow() {
                            row_entry_map_clone.borrow_mut().remove(widget);

                            // Find and remove the corresponding ListBoxRow
                            let mut child = list_box_clone.first_child();
                            while let Some(row) = child {
                                if let Some(list_row) = row.downcast_ref::<gtk4::ListBoxRow>() {
                                    if let Some(row_child) = list_row.child() {
                                        if &row_child == widget {
                                            list_box_clone.remove(list_row);
                                            break;
                                        }
                                    }
                                }
                                child = row.next_sibling();
                            }
                        }

                        // Remove from all_entries
                        let mut entries = all_entries_clone.borrow_mut();
                        entries.retain(|e| e.path != entry.path);

                        // Clear current selection
                        *current_entry_clone.borrow_mut() = None;
                        *current_row_widget_clone.borrow_mut() = None;

                        // Hide editor and show welcome message
                        editor_box_clone.set_visible(false);
                        details_label_clone.set_visible(true);
                    }
                    Err(e) => {
                        eprintln!("Error deleting file: {}", e);
                    }
                }
            }
        });

        MainWindow { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn update_icon_preview(icon_preview: &Image, icon_name: &str) {
    if !icon_name.is_empty() {
        if std::path::Path::new(icon_name).exists() {
            icon_preview.set_from_file(Some(icon_name));
        } else {
            icon_preview.set_icon_name(Some(icon_name));
        }
    } else {
        icon_preview.set_icon_name(Some("application-x-executable"));
    }
}

fn update_row_icon(row_widget: &Rc<RefCell<Option<gtk4::Widget>>>, icon_name: &str) {
    if let Some(ref widget) = *row_widget.borrow() {
        // The widget is a GtkBox (horizontal) with icon as first child
        if let Some(row_box) = widget.downcast_ref::<GtkBox>() {
            if let Some(icon_widget) = row_box.first_child() {
                if let Some(image) = icon_widget.downcast_ref::<Image>() {
                    // Update the icon
                    if !icon_name.is_empty() {
                        if std::path::Path::new(icon_name).exists() {
                            image.set_from_file(Some(icon_name));
                        } else {
                            image.set_icon_name(Some(icon_name));
                        }
                    } else {
                        image.set_icon_name(Some("application-x-executable"));
                    }
                }
            }
        }
    }
}

fn clear_list_box(list: &ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
}

fn add_mime_association(
    mime_type: &str,
    extension: Option<&str>,
    state: &Rc<RefCell<Vec<String>>>,
    extensions_state: &Rc<RefCell<HashMap<String, String>>>,
    list: &ListBox,
) -> bool {
    if state
        .borrow()
        .iter()
        .any(|existing| existing == mime_type)
    {
        return false;
    }

    state.borrow_mut().push(mime_type.to_string());
    if let Some(ext) = extension {
        extensions_state
            .borrow_mut()
            .insert(mime_type.to_string(), ext.to_string());
    }

    let ext_for_row = extension
        .map(|value| value.to_string())
        .or_else(|| extensions_state.borrow().get(mime_type).cloned())
        .filter(|value| !value.is_empty());

    append_mime_row(
        list,
        mime_type,
        ext_for_row.as_deref(),
        state.clone(),
        extensions_state.clone(),
    );
    true
}

fn row_string_data(row: &gtk4::ListBoxRow, key: &str) -> Option<String> {
    unsafe { row.data::<String>(key).map(|ptr| ptr.as_ref().clone()) }
}

fn add_mime_from_row(
    row: &gtk4::ListBoxRow,
    state: &Rc<RefCell<Vec<String>>>,
    list: &ListBox,
    current_entry: &Rc<RefCell<Option<DesktopEntry>>>,
    known_map: &Rc<RefCell<HashMap<String, String>>>,
    extensions_state: &Rc<RefCell<HashMap<String, String>>>,
) {
    if let Some(mime_value) = row_string_data(row, "mime-type") {
        let extension_value = row_string_data(row, "extension");
        let extension_opt = extension_value
            .as_ref()
            .and_then(|value| if value.is_empty() { None } else { Some(value.as_str()) });

        let mut map = known_map.borrow_mut();
        if let Some(ext_str) = extension_opt {
            map.entry(mime_value.clone()).or_insert_with(|| ext_str.to_string());
        }

        let resolved_extension = extension_opt.or_else(|| map.get(&mime_value).map(|s| s.as_str()));

        if add_mime_association(
            &mime_value,
            resolved_extension,
            state,
            extensions_state,
            list,
        ) {
            if let Some(ref mut entry) = *current_entry.borrow_mut() {
                entry.mime_types = state.borrow().clone();
                entry.mime_extensions = extensions_state.borrow().clone();
            }

            let display_extension = resolved_extension.unwrap_or("-");
            println!(
                "Added file association '{}' ({})",
                mime_value,
                display_extension
            );
        } else {
            println!(
                "File association '{}' already exists",
                mime_value
            );
        }
    }
}

fn try_add_manual_mime(
    entry: &Entry,
    state: &Rc<RefCell<Vec<String>>>,
    list: &ListBox,
    current_entry: &Rc<RefCell<Option<DesktopEntry>>>,
    known_map: &Rc<RefCell<HashMap<String, String>>>,
    extensions_state: &Rc<RefCell<HashMap<String, String>>>,
) {
    let input = entry.text().to_string();

    match resolve_mime_from_input(&input) {
        Some((mime, display_hint)) => {
            let extension_opt = if display_hint.is_empty() {
                None
            } else {
                Some(display_hint.as_str())
            };

            let mut map = known_map.borrow_mut();
            if let Some(ext_str) = extension_opt {
                map.entry(mime.clone()).or_insert_with(|| ext_str.to_string());
            }
            let resolved_extension = extension_opt
                .or_else(|| map.get(&mime).map(|value| value.as_str()));

            if add_mime_association(
                &mime,
                resolved_extension,
                state,
                extensions_state,
                list,
            ) {
                if let Some(ref mut entry_ref) = *current_entry.borrow_mut() {
                    entry_ref.mime_types = state.borrow().clone();
                    entry_ref.mime_extensions = extensions_state.borrow().clone();
                }

                println!(
                    "Added file association '{}' ({})",
                    mime,
                    resolved_extension.unwrap_or("-")
                );
            } else {
                println!(
                    "File association '{}' already exists",
                    mime
                );
            }
        }
        None => {
            if !input.trim().is_empty() {
                println!(
                    "Unable to determine MIME type from '{}'",
                    input.trim()
                );
            }
        }
    }

    entry.set_text("");
}

fn append_mime_row(
    list: &ListBox,
    mime_type: &str,
    extension: Option<&str>,
    state: Rc<RefCell<Vec<String>>>,
    extensions_state: Rc<RefCell<HashMap<String, String>>>,
) {
    let row = gtk4::ListBoxRow::new();
    row.set_selectable(false);
    row.set_activatable(false);

    let row_box = GtkBox::new(Orientation::Horizontal, 5);

    let extension_owned = extension.unwrap_or("").to_string();
    let ext_display = if extension_owned.is_empty() {
        "—"
    } else {
        extension_owned.as_str()
    };

    let ext_label = Label::new(Some(ext_display));
    ext_label.add_css_class("monospace");
    ext_label.set_width_chars(6);
    ext_label.set_halign(gtk4::Align::Start);

    let label = Label::new(Some(mime_type));
    label.set_halign(gtk4::Align::Start);
    label.set_hexpand(true);
    label.set_wrap(true);

    let remove_button = Button::with_label("Remove");
    remove_button.add_css_class("flat");
    remove_button.set_halign(gtk4::Align::End);

    row_box.append(&ext_label);
    row_box.append(&label);
    row_box.append(&remove_button);
    row.set_child(Some(&row_box));
    list.append(&row);

    unsafe {
        row.set_data("mime-type", mime_type.to_string());
        row.set_data("extension", extension_owned.clone());
    }

    let mime_value = mime_type.to_string();
    let state_clone = state.clone();
    let extensions_state_clone = extensions_state.clone();
    let list_weak = Downgrade::downgrade(&list);
    let row_weak = Downgrade::downgrade(&row);

    remove_button.connect_clicked(move |_| {
        state_clone
            .borrow_mut()
            .retain(|existing| existing != &mime_value);
        extensions_state_clone.borrow_mut().remove(&mime_value);

        if let (Some(list_box), Some(row_widget)) = (list_weak.upgrade(), row_weak.upgrade()) {
            list_box.remove(&row_widget);
        }
    });
}

fn resolve_mime_from_input(input: &str) -> Option<(String, String)> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.contains('/') {
        return Some((trimmed.to_string(), String::new()));
    }

    let sanitized = trimmed.trim_start_matches('.');
    if sanitized.is_empty() {
        return None;
    }

    let filename = format!("dummy.{}", sanitized);
    let path = std::path::Path::new(&filename);
    let (content_type, uncertain) = gio::content_type_guess(Some(path), &[]);

    if let Some(mime) = gio::content_type_get_mime_type(content_type.as_str()) {
        return Some((mime.to_string(), format!(".{}", sanitized)));
    }

    if !uncertain {
        return Some((content_type.to_string(), format!(".{}", sanitized)));
    }

    Some((
        format!("application/x-{}", sanitized.to_lowercase()),
        format!(".{}", sanitized),
    ))
}

fn build_mime_extension_map(choices: &[MimeChoice]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for choice in choices {
        map.entry(choice.mime_type.clone())
            .or_insert_with(|| choice.extension.clone());
    }
    map
}

fn load_known_extensions() -> Vec<MimeChoice> {
    let mut map: HashMap<String, (String, u32)> = HashMap::new();

    for path in mime_database_paths() {
        if let Ok(file) = File::open(&path) {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }

                let mut parts = trimmed.splitn(3, ':');
                let weight_str = parts.next().unwrap_or("0");
                let mime = parts.next().unwrap_or("").trim();
                let pattern = parts.next().unwrap_or("").trim();

                if mime.is_empty() || pattern.is_empty() {
                    continue;
                }

                if let Some(extension) = extract_extension_from_pattern(pattern) {
                    let weight = weight_str.parse::<u32>().unwrap_or(0);
                    let entry = map
                        .entry(extension.clone())
                        .or_insert_with(|| (mime.to_string(), weight));

                    if weight >= entry.1 {
                        *entry = (mime.to_string(), weight);
                    }
                }
            }
        }
    }

    let mut choices = Vec::new();
    for (extension, (mime, _weight)) in map {
        let description = mime_description(&mime);
        choices.push(MimeChoice {
            extension: format!(".{}", extension),
            mime_type: mime,
            description,
        });
    }

    choices.sort_by(|a, b| a.extension.cmp(&b.extension));
    choices
}

fn mime_database_paths() -> Vec<String> {
    let mut paths = Vec::new();

    if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
        if !xdg_data_home.is_empty() {
            push_unique_path(&mut paths, format!(
                "{}/mime/globs2",
                xdg_data_home.trim_end_matches('/')
            ));
        }
    }

    if let Ok(home) = env::var("HOME") {
        push_unique_path(&mut paths, format!("{}/.local/share/mime/globs2", home));
    }

    if let Ok(xdg_dirs) = env::var("XDG_DATA_DIRS") {
        for dir in xdg_dirs.split(':') {
            let trimmed = dir.trim();
            if !trimmed.is_empty() {
                push_unique_path(&mut paths, format!("{}/mime/globs2", trimmed.trim_end_matches('/')));
            }
        }
    }

    push_unique_path(&mut paths, String::from("/usr/local/share/mime/globs2"));
    push_unique_path(&mut paths, String::from("/usr/share/mime/globs2"));

    paths
}

fn push_unique_path(paths: &mut Vec<String>, candidate: String) {
    if !paths.contains(&candidate) {
        paths.push(candidate);
    }
}

fn extract_extension_from_pattern(pattern: &str) -> Option<String> {
    if !pattern.starts_with("*.") {
        return None;
    }

    let ext = &pattern[2..];
    if ext.is_empty() {
        return None;
    }

    if ext.chars().any(|c| matches!(c, '*' | '?' | '[' | ']' | '!')) {
        return None;
    }

    Some(ext.to_lowercase())
}

fn mime_description(mime: &str) -> String {
    if let Some(content_type) = gio::content_type_from_mime_type(mime) {
        let description = gio::content_type_get_description(content_type.as_str());
        let value = description.to_string();
        if !value.is_empty() {
            return value;
        }
    }

    mime.to_string()
}

fn show_mime_selection_dialog(
    parent: &ApplicationWindow,
    known_choices: Rc<Vec<MimeChoice>>,
    known_map: Rc<RefCell<HashMap<String, String>>>,
    mime_state: Rc<RefCell<Vec<String>>>,
    extension_state: Rc<RefCell<HashMap<String, String>>>,
    mime_list: &ListBox,
    current_entry: Rc<RefCell<Option<DesktopEntry>>>,
) {
    let dialog = gtk4::Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title("Add File Association")
        .default_width(520)
        .default_height(480)
        .build();

    dialog.add_button("Close", ResponseType::Close);

    let content = dialog.content_area();
    content.set_spacing(8);
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);

    let filter_entry = Entry::builder()
        .placeholder_text("Search by extension, description, or MIME type")
        .build();
    content.append(&filter_entry);

    let scrolled = ScrolledWindow::builder()
        .min_content_height(260)
        .hexpand(true)
        .vexpand(true)
        .build();

    let list_box = ListBox::new();
    list_box.set_selection_mode(gtk4::SelectionMode::Browse);
    scrolled.set_child(Some(&list_box));
    content.append(&scrolled);

    if known_choices.is_empty() {
        let empty_label = Label::new(Some("No known file types available. Use manual input below."));
        empty_label.set_halign(gtk4::Align::Start);
        empty_label.add_css_class("dim-label");
        content.append(&empty_label);
    }

    let select_box = GtkBox::new(Orientation::Horizontal, 6);
    select_box.set_hexpand(true);

    let hint_label = Label::new(Some("Double-click a row or use \"Add Selected\"."));
    hint_label.set_halign(gtk4::Align::Start);
    hint_label.set_hexpand(true);
    hint_label.add_css_class("dim-label");
    select_box.append(&hint_label);

    let add_selected_button = Button::with_label("Add Selected");
    add_selected_button.set_sensitive(false);
    add_selected_button.set_halign(gtk4::Align::End);
    select_box.append(&add_selected_button);

    content.append(&select_box);

    let manual_box = GtkBox::new(Orientation::Horizontal, 6);
    manual_box.set_hexpand(true);

    let manual_entry = Entry::builder()
        .placeholder_text("Enter extension (e.g. .txt) or MIME type")
        .build();
    manual_entry.set_hexpand(true);
    manual_box.append(&manual_entry);

    let manual_add_button = Button::with_label("Add From Text");
    manual_box.append(&manual_add_button);

    content.append(&manual_box);

    for choice in known_choices.iter() {
        let row = gtk4::ListBoxRow::new();
        row.set_activatable(true);
        row.set_selectable(true);

        unsafe {
            row.set_data("mime-type", choice.mime_type.clone());
            row.set_data("extension", choice.extension.clone());
            row.set_data("description", choice.description.clone());
        }

        let row_box = GtkBox::new(Orientation::Horizontal, 10);
        row_box.set_hexpand(true);

        let ext_label = Label::new(Some(&choice.extension));
        ext_label.add_css_class("monospace");
        ext_label.set_width_chars(8);
        ext_label.set_halign(gtk4::Align::Start);

        let description_label = Label::new(Some(&choice.description));
        description_label.set_halign(gtk4::Align::Start);
        description_label.set_hexpand(true);
        description_label.set_wrap(true);

        let mime_label = Label::new(Some(choice.mime_type.as_str()));
        mime_label.add_css_class("dim-label");
        mime_label.set_halign(gtk4::Align::End);

        row_box.append(&ext_label);
        row_box.append(&description_label);
        row_box.append(&mime_label);

        row.set_child(Some(&row_box));
        list_box.append(&row);
    }

    let list_box_for_filter = list_box.clone();
    filter_entry.connect_changed(move |entry| {
        let query = entry.text().to_lowercase();
        let mut child = list_box_for_filter.first_child();
        while let Some(widget) = child {
            if let Some(row) = widget.downcast_ref::<gtk4::ListBoxRow>() {
                let extension = row_string_data(row, "extension")
                    .unwrap_or_default()
                    .to_lowercase();
                let description = row_string_data(row, "description")
                    .unwrap_or_default()
                    .to_lowercase();
                let mime = row_string_data(row, "mime-type")
                    .unwrap_or_default()
                    .to_lowercase();

                let matches = query.is_empty()
                    || extension.contains(&query)
                    || description.contains(&query)
                    || mime.contains(&query);
                row.set_visible(matches);
            }
            child = widget.next_sibling();
        }
    });

    let add_selected_button_clone = add_selected_button.clone();
    list_box.connect_row_selected(move |_, row| {
        add_selected_button_clone.set_sensitive(row.is_some());
    });

    let state_for_activation = mime_state.clone();
    let list_for_activation = mime_list.clone();
    let entry_for_activation = current_entry.clone();
    let map_for_activation = known_map.clone();
    let extensions_for_activation = extension_state.clone();
    list_box.connect_row_activated(move |_, row| {
        add_mime_from_row(
            row,
            &state_for_activation,
            &list_for_activation,
            &entry_for_activation,
            &map_for_activation,
            &extensions_for_activation,
        );
    });

    let list_for_button = list_box.clone();
    let state_for_button = mime_state.clone();
    let mime_list_for_button = mime_list.clone();
    let entry_for_button = current_entry.clone();
    let map_for_button = known_map.clone();
    let extensions_for_button = extension_state.clone();
    add_selected_button.connect_clicked(move |_| {
        if let Some(row) = list_for_button.selected_row() {
            add_mime_from_row(
                &row,
                &state_for_button,
                &mime_list_for_button,
                &entry_for_button,
                &map_for_button,
                &extensions_for_button,
            );
        }
    });

    let state_for_manual = mime_state.clone();
    let mime_list_for_manual = mime_list.clone();
    let entry_for_manual = current_entry.clone();
    let map_for_manual = known_map.clone();
    let extensions_for_manual = extension_state.clone();
    let manual_entry_button = manual_entry.clone();
    manual_add_button.connect_clicked(move |_| {
        try_add_manual_mime(
            &manual_entry_button,
            &state_for_manual,
            &mime_list_for_manual,
            &entry_for_manual,
            &map_for_manual,
            &extensions_for_manual,
        );
    });

    let state_for_entry = mime_state.clone();
    let mime_list_for_entry = mime_list.clone();
    let entry_for_entry = current_entry.clone();
    let map_for_entry = known_map.clone();
    let extensions_for_entry = extension_state.clone();
    manual_entry.connect_activate(move |entry| {
        try_add_manual_mime(
            entry,
            &state_for_entry,
            &mime_list_for_entry,
            &entry_for_entry,
            &map_for_entry,
            &extensions_for_entry,
        );
    });

    dialog.connect_response(|dialog, _| dialog.close());
    dialog.show();
}

fn create_list_row(name: &str, comment: &str, icon_name: &str) -> GtkBox {
    let row_box = GtkBox::new(Orientation::Horizontal, 10);
    row_box.set_margin_start(5);
    row_box.set_margin_end(5);
    row_box.set_margin_top(5);
    row_box.set_margin_bottom(5);

    // Icon
    let icon = if !icon_name.is_empty() {
        // Try loading as icon name first, then as file path
        if std::path::Path::new(icon_name).exists() {
            Image::from_file(icon_name)
        } else {
            Image::from_icon_name(icon_name)
        }
    } else {
        Image::from_icon_name("application-x-executable")
    };
    icon.set_pixel_size(48);
    row_box.append(&icon);

    // Text container
    let text_box = GtkBox::new(Orientation::Vertical, 2);
    text_box.set_hexpand(true);

    let name_label = Label::new(Some(name));
    name_label.set_halign(gtk4::Align::Start);
    name_label.add_css_class("title-4");

    let comment_label = Label::new(Some(comment));
    comment_label.set_halign(gtk4::Align::Start);
    comment_label.add_css_class("dim-label");
    comment_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);

    text_box.append(&name_label);
    text_box.append(&comment_label);

    row_box.append(&text_box);

    row_box
}

fn can_write_file(path: &std::path::Path) -> bool {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    // If file doesn't exist yet (new entry), check if we can write to parent directory
    if !path.exists() {
        if let Some(parent) = path.parent() {
            return parent.exists() && can_write_file(parent);
        }
        return false;
    }

    // Check file permissions
    match fs::metadata(path) {
        Ok(metadata) => {
            let permissions = metadata.permissions();
            let mode = permissions.mode();

            // Check if owner has write permission (bit 7)
            // For a more thorough check, we'd also verify the current user is the owner
            let owner_write = (mode & 0o200) != 0;

            // Also check if file is actually writable by attempting to open it
            let writable = std::fs::OpenOptions::new()
                .write(true)
                .open(path)
                .is_ok();

            owner_write && writable
        }
        Err(_) => false,
    }
}
