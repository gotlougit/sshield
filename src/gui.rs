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
