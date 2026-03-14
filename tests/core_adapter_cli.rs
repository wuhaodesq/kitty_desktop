use kitty_desktop::normalize_extra_args;

#[test]
fn normalize_extra_args_strips_remainder_separator() {
    let args = vec!["--".to_string(), "--single-instance".to_string()];
    assert_eq!(normalize_extra_args(&args), vec!["--single-instance"]);
}
