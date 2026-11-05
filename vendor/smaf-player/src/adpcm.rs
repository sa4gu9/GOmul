// decode yamaha adpcm-b
// adapted to rust from https://github.com/superctr/adpcm/blob/master/ymb_codec.c (Unlicense)

use alloc::vec::Vec;

static STEP_TABLE: [u8; 8] = [57, 57, 57, 57, 77, 102, 128, 153];

struct DecodeContext {
    history: i16,
    step_size: u16,
}

fn ymb_step(step: u8, context: &mut DecodeContext) -> i16 {
    let sign = step & 8;
    let delta = step & 7;
    let diff = ((1 + ((delta as u32) << 1)) * (context.step_size as u32)) >> 3;
    let mut newval = context.history as i32;
    let nstep = (((STEP_TABLE[delta as usize] as u32) * (context.step_size as u32)) >> 6) as u16;
    if sign > 0 {
        newval -= diff as i32;
    } else {
        newval += diff as i32;
    }
    context.step_size = u16::clamp(nstep, 127, 24576);
    newval = i32::clamp(newval, -32768, 32767);
    context.history = newval as i16;

    newval as i16
}

pub fn decode_adpcm(data: &[u8]) -> Vec<i16> {
    let mut result = Vec::new();
    let mut context = DecodeContext { history: 0, step_size: 127 };

    // SMAF stores the earlier sample in the low nibble of each byte.
    for &byte in data {
        result.push(ymb_step(byte & 0x0f, &mut context));
        result.push(ymb_step(byte >> 4, &mut context));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::decode_adpcm;

    #[test]
    fn decodes_low_nibble_before_high_nibble() {
        // 0x07 then 0x00 in the low nibbles: the first sample takes the large positive step.
        let samples = decode_adpcm(&[0x07, 0x08]);
        assert_eq!(samples.len(), 4);
        assert!(samples[0] > 0);
        assert_eq!(samples[0], (15 * 127 / 8) as i16);
    }
}
