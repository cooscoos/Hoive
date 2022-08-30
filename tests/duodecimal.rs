use hoive::maths::funcs;

#[test]
fn duodecimal_from_decimal() {
    let numbers = [5, 10, 16, 22, 28, 34, 143 as usize];

    let result = numbers
        .into_iter()
        .map(|v| funcs::decimal_to_duo(v))
        .collect::<Vec<String>>();

    let expected = vec![
        "05".to_string(),
        "0x".to_string(),
        "14".to_string(),
        "1x".to_string(),
        "24".to_string(),
        "2x".to_string(),
        "yy".to_string(),
    ];

    assert_eq!(expected, result);
}

#[test]
fn duodecimal_to_decimal() {
    let numbers = vec![
        "05".to_string(),
        "0x".to_string(),
        "14".to_string(),
        "1x".to_string(),
        "24".to_string(),
        "2x".to_string(),
        "yy".to_string(),
    ];

    let result = numbers
        .into_iter()
        .map(|d| funcs::duo_to_decimal(d))
        .collect::<Vec<usize>>();

    let expected = vec![5, 10, 16, 22, 28, 34, 143 as usize];

    assert_eq!(expected, result);
}
