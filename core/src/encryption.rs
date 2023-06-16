const KEY: &str = "ZorbakOwnsYou";


pub fn decrypt(text: &str) -> Result<String, ()> {
    let mut result = String::new();
    for index in (0..text.len()).step_by(4) {
        let first_radix = text.get(index..index + 2);
        if first_radix.is_none() {
            return Err(());
        }
        let first_radix = first_radix.unwrap();

        let first_part = u32::from_str_radix(first_radix, 30);
        if first_part.is_err() {
            return Err(());
        }
        let first_part = first_part.unwrap();

        let second_radix = text.get(index + 2..index + 4);
        if second_radix.is_none() {
            return Err(());
        }
        let second_radix = second_radix.unwrap();

        let second_part = u32::from_str_radix(second_radix, 30);
        if second_part.is_err() {
            return Err(());
        }
        let second_part = second_part.unwrap();

        let key_value = KEY.chars().nth((index / 4) % KEY.len()).unwrap() as u32;

        let letter = char::from_u32(first_part - second_part - key_value);
        if letter.is_none() {
            return Err(());
        }
        let letter = letter.unwrap();

        result.push(letter);
    }

    Ok(result)
}
