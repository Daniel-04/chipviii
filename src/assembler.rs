use std::collections::HashMap;

pub struct Assembler {
    labels: HashMap<String, u16>,
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
        }
    }

    pub fn assemble(&mut self, source: &str) -> Result<Vec<u8>, String> {
        let lines: Vec<&str> = source
            .lines()
            .map(|l| l.split(';').next().unwrap_or("").trim())
            .filter(|l| !l.is_empty())
            .collect();

        // labels
        let mut current_pc = 0x200;
        for line in &lines {
            if line.ends_with(':') {
                let label = line[..line.len() - 1].to_string();
                self.labels.insert(label, current_pc);
            } else {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts[0].to_uppercase() == "DB" {
                    current_pc += 1; // bytes
                } else {
                    current_pc += 2; // opcodes
                }
            }
        }

        // instructions
        let mut binary = Vec::new();
        for line in &lines {
            if line.ends_with(':') {
                continue;
            }

            let parts: Vec<&str> = line
                .split(|c| c == ' ' || c == ',')
                .filter(|s| !s.is_empty())
                .collect();

            let mnemonic = parts[0].to_uppercase();

            if mnemonic == "DB" {
                let val = self.parse_u8(parts[1])?;
                binary.push(val);
            } else {
                if binary.len() % 2 != 0 {
                    binary.push(0x00); // align instruction
                }
                let opcode = self.encode_instruction(&parts)?;
                binary.push((opcode >> 8) as u8);
                binary.push((opcode & 0xFF) as u8);
            }
        }

        Ok(binary)
    }

    fn encode_instruction(&self, parts: &[&str]) -> Result<u16, String> {
        let mnemonic = parts[0].to_uppercase();
        match mnemonic.as_str() {
            "CLS" => Ok(0x00E0),
            "RET" => Ok(0x00EE),
            "JP" => Ok(0x1000 | (self.resolve_addr(parts[1])? & 0x0FFF)),
            "LD" => {
                let dest = parts[1].to_uppercase();
                if dest == "I" {
                    Ok(0xA000 | (self.resolve_addr(parts[2])? & 0x0FFF))
                } else if dest.starts_with('V') {
                    let x = self.parse_reg(&dest)?;
                    if parts[2].to_uppercase().starts_with('V') {
                        let y = self.parse_reg(parts[2])?;
                        Ok(0x8000 | (x << 8) | (y << 4))
                    } else {
                        let val = self.parse_u8(parts[2])?;
                        Ok(0x6000 | (x << 8) | val as u16)
                    }
                } else {
                    Err("Invalid LD".into())
                }
            }
            "ADD" => {
                let x = self.parse_reg(parts[1])?;
                let val = self.parse_u8(parts[2])?;
                Ok(0x7000 | (x << 8) | val as u16)
            }
            "DRW" => {
                let x = self.parse_reg(parts[1])?;
                let y = self.parse_reg(parts[2])?;
                let n = self.parse_u8(parts[3])? as u16;
                Ok(0xD000 | (x << 8) | (y << 4) | (n & 0x000F))
            }
            _ => Err(format!("Unknown: {}", mnemonic)),
        }
    }

    fn resolve_addr(&self, s: &str) -> Result<u16, String> {
        self.labels
            .get(s)
            .cloned()
            .ok_or_else(|| "".to_string())
            .or_else(|_| self.parse_u16(s))
    }

    fn parse_reg(&self, s: &str) -> Result<u16, String> {
        u16::from_str_radix(&s[1..], 16).map_err(|_| "Reg err".into())
    }

    fn parse_u8(&self, s: &str) -> Result<u8, String> {
        self.parse_u16(s).map(|v| v as u8)
    }

    fn parse_u16(&self, s: &str) -> Result<u16, String> {
        if s.starts_with("0x") {
            u16::from_str_radix(&s[2..], 16)
        } else {
            s.parse::<u16>()
        }
        .map_err(|_| "Num err".into())
    }
}
