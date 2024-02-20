use crate::core::msg::mapper;

pub fn new(channel: String, data: String, compress: bool) -> Vec<u8> {
    let mut plugin_message: Vec<u8> = Vec::new();
    if compress {
        plugin_message.push(0x00);
    }
    plugin_message.push(mapper::PLUGIN_MESSAGE);
    plugin_message.push(channel.len() as u8);
    plugin_message.extend(channel.as_bytes());
    plugin_message.push(data.len() as u8);
    plugin_message.extend(data.as_bytes());
    plugin_message = [vec![plugin_message.len() as u8], plugin_message].concat();
    plugin_message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let channel = "minecraft:brand".to_string();
        let data = "Minecraft-Console-Client/1.20.2".to_string();
        let compress = true;
        let plugin_message = new(channel, data, compress);
        assert_eq!(
            plugin_message,
            vec![
                0x32, 0x00, 0x0d, 0x0f, 0x6d, 0x69, 0x6e, 0x65, 0x63, 0x72, 0x61, 0x66, 0x74, 0x3a,
                0x62, 0x72, 0x61, 0x6e, 0x64, 0x1f, 0x4d, 0x69, 0x6e, 0x65, 0x63, 0x72, 0x61, 0x66,
                0x74, 0x2d, 0x43, 0x6f, 0x6e, 0x73, 0x6f, 0x6c, 0x65, 0x2d, 0x43, 0x6c, 0x69, 0x65,
                0x6e, 0x74, 0x2f, 0x31, 0x2e, 0x32, 0x30, 0x2e, 0x32
            ]
        );
    }
}
