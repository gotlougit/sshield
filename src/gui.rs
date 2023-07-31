use dialog::{Choice, DialogBox};

pub fn confirm_request(msg: &str) -> bool {
    let choice = dialog::Question::new(msg)
        .title("sshield")
        .show()
        .unwrap_or(Choice::No);
    match choice {
        Choice::Yes => true,
        _ => false,
    }
}

pub fn get_db_pass() -> String {
    dialog::Password::new("Enter database password: ")
        .title("Password")
        .show()
        .unwrap()
        .unwrap_or(String::new())
}

pub fn get_new_db_pass() -> String {
    dialog::Password::new("Enter new database password: ")
        .title("sshield- Change password")
        .show()
        .unwrap()
        .unwrap_or(String::new())
}
