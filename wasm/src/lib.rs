use chip8_rs::chip8::Chip8;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
struct WasmChip8(Chip8);

#[wasm_bindgen]
impl WasmChip8 {
    pub async fn new(url: &str) -> Result<WasmChip8, JsError> {
        let response = reqwest::get(url).await?.bytes().await?;
        let bytes: Vec<u8> = response.into();
        let mut chip8 = Chip8::new();
        match chip8.load_from_bin(&bytes) {
            Err(str) => {
                return Err(JsError::new(str));
            }
            _ => (),
        }
        Ok(Self(chip8))
    }

    pub fn get_ram(&self) -> Vec<u8> {
        self.0.ram.to_vec()
    }

    pub fn get_screen(&self) -> Vec<u8> {
        let mut ret = Vec::new();
        self.0.screen.iter().for_each(|r| ret.extend_from_slice(r));

        ret
    }

    pub fn get_pc(&self) -> usize {
        self.0.pc
    }

    pub async fn cycle(&mut self, input: &[usize]) {
        let mut keypad = [false; 16];
        for i in 0..16 {
            keypad[i] = input[i] == 1;
        }
        self.0.cycle(keypad);
    }
}
