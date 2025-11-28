use notify_rust::Notification;

pub fn error(title: &str, message: &str) {
    eprintln!("[ERROR] {}: {}", title, message);

    let _ = Notification::new()
    .summary(title)
    .body(message)
    .show();
}