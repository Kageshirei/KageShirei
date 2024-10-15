use bytes::Bytes;

use crate::encoder::Encoder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Base32Encoder {
    alphabet: &'static [u8],
}

impl Default for Base32Encoder {
    fn default() -> Self {
        Self {
            alphabet: b"abcdefghijklmnopqrstuvwxyz234567",
        }
    }
}

impl Encoder for Base32Encoder {
    fn encode(&self, data: Bytes) -> String {
        let mut bits = 0u32;
        let mut bit_count = 0;
        let mut output = Vec::new();

        for byte in data.to_vec() {
            bits = (bits << 8) | byte as u32;
            bit_count += 8;

            while bit_count >= 5 {
                let index = ((bits >> (bit_count - 5)) & 0x1f) as usize;
                output.push(self.alphabet[index]);
                bit_count -= 5;
            }
        }

        if bit_count > 0 {
            let index = ((bits << (5 - bit_count)) & 0x1f) as usize;
            output.push(self.alphabet[index]);
        }

        output.iter().map(|c| *c as char).collect::<String>()
    }

    fn decode(&self, data: &str) -> Result<Bytes, String> {
        let mut bits = 0u32;
        let mut bit_count = 0;
        let mut output = Vec::new();

        for byte in data.bytes() {
            let value = match byte {
                b'a' ..= b'z' => byte - b'a',
                b'2' ..= b'7' => byte - b'2' + 26,
                _ => return Err("Invalid character in input".to_owned()),
            } as u32;

            bits = (bits << 5) | value;
            bit_count += 5;

            if bit_count >= 8 {
                output.push((bits >> (bit_count - 8)) as u8);
                bit_count -= 8;
            }
        }

        Ok(Bytes::from(output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::Encoder as _;

    #[test]
    fn test_encode() {
        let data = Bytes::from_static(b"Hello, World!");
        let encoded = Base32Encoder::default().encode(data);
        assert_eq!(encoded, "jbswy3dpfqqfo33snrscc");
    }

    #[test]
    fn test_decode() {
        let data = "jbswy3dpfqqfo33snrscc";
        let decoded = Base32Encoder::default().decode(data).unwrap();
        assert_eq!(decoded, Bytes::from_static(b"Hello, World!"));
    }
}
