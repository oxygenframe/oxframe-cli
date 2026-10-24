//! A terminal window: a command is typed, then its output appears.
//!
//! `blocks::terminal(view, "cargo install oxframe-cli", &["Installed oxframe 0.23.0"]).await`

use oxframe::prelude::*;

const WIDTH: f64 = 1200.0;
const LINE: f64 = 56.0;

/// Types `command` after a `$` prompt, then shows each line of `output`.
/// Returns the block's group.
pub async fn terminal(view: View, command: &str, output: &[&str]) -> Group {
    let height = 200.0 + LINE * output.len() as f64;
    let group = view.add(Group::new().opacity(0.0).name("terminal"));
    let _ = group.add(
        Rect::new()
            .width(WIDTH)
            .height(height)
            .radius(20.0)
            .fill(Surface)
            .name("terminal_window"),
    );
    let top = -height / 2.0;
    for (i, color) in [Danger, Warning, Success].into_iter().enumerate() {
        let _ = group.add(
            Circle::new()
                .size(18.0)
                .fill(color)
                .x(-WIDTH / 2.0 + 40.0 + 30.0 * i as f64)
                .y(top + 36.0),
        );
    }
    let left = -WIDTH / 2.0 + 48.0;
    let first = top + 110.0;
    let _ = group.add(
        Txt::new()
            .text("$")
            .code()
            .fill(Accent)
            .anchor((-1.0, 0.0))
            .x(left)
            .y(first),
    );
    let typed = group.add(
        Txt::new()
            .text("")
            .code()
            .anchor((-1.0, 0.0))
            .x(left + 40.0)
            .y(first)
            .name("terminal_command"),
    );
    group.fade_in().await;
    typed.text().to(command, 0.04 * command.chars().count() as f64).await;
    wait_for(0.3).await;
    for (i, &line) in output.iter().enumerate() {
        let out = group.add(
            Txt::new()
                .text(line)
                .code()
                .fill(Muted)
                .anchor((-1.0, 0.0))
                .x(left)
                .y(first + LINE * (i + 1) as f64)
                .opacity(0.0),
        );
        out.fade_in().await;
    }
    group
}
