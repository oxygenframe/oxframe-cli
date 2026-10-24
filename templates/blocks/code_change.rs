//! A piece of code that changes into its new version.
//!
//! `blocks::code_change(view, "rust", "let x = 1;", "let x = 2;").await`

use oxframe::prelude::*;

/// Shows `before`, then diffs it into `after`: kept tokens slide, removed
/// ones fade, new ones appear. Returns the block's group.
pub async fn code_change(view: View, language: &str, before: &str, after: &str) -> Group {
    let group = view.add(Group::new().opacity(0.0).name("code_change"));
    let code = group.add(
        Code::new()
            .code(before)
            .language(language)
            .font_size(48.0)
            .name("code_change_code"),
    );
    group.fade_in().await;
    wait_for(0.8).await;
    code.code().to(after, 1.2).await;
    group
}
