use anyhow::anyhow;

const KEY: &str = "ZorbakOwnsYou";

pub fn decrypt(text: &str) -> Result<String, anyhow::Error> {
    let mut result = String::new();
    for index in (0..text.len()).step_by(4) {
        let first_radix = text
            .get(index..index + 2)
            .ok_or(anyhow!("Index out of range"))?;
        let first_part = u32::from_str_radix(first_radix, 30)?;

        let second_radix = text
            .get(index + 2..index + 4)
            .ok_or(anyhow!("Index out of range"))?;
        let second_part = u32::from_str_radix(second_radix, 30)?;

        let key_value = KEY
            .chars()
            .nth((index / 4) % KEY.len())
            .ok_or(anyhow!("Index out of range"))? as u32;
        let letter = char::from_u32(first_part - second_part - key_value)
            .ok_or(anyhow!("Failed to convert u32 to char"))?;

        result.push(letter);
    }

    Ok(result)
}
