use crypto::hash::k12_bytes;
use crypto::qubic_identities::{get_public_key_from_identity, verify};
use crate::{ARBITRATOR, NUMBER_COMPUTORS, QUORUM_MINIMUM_VOTES, TICK_TYPE};
use crate::computor::{BroadcastComputors, ComputorPubKey};
use crate::tick::Tick;

pub fn get_quorum_votes(bc: &BroadcastComputors, ticks: &Vec<Tick>) -> Result<bool, String> {
    const num_vote_flags: usize = (NUMBER_COMPUTORS + 7) / 8;
    let arbitrator: [u8; 32] = get_public_key_from_identity(&String::from(ARBITRATOR)).unwrap();
    let mut vote_flags: [u8; num_vote_flags] = [0; num_vote_flags];
    if ticks.len() == 0 as usize {
        return Ok(false);
    }
    println!("Getting Quorum Votes For Tick {}", ticks.first().unwrap().tick);
    for (idx, vote) in ticks.iter().enumerate() {
        let mut tick = vote.clone();
        tick.computor_index = tick.computor_index ^ TICK_TYPE as u16;
        let digest: [u8; 32] = tick.hash();
        tick.computor_index = tick.computor_index ^ TICK_TYPE as u16;
        let pub_key = &bc.pub_keys[tick.computor_index as usize];
        if !verify(&pub_key.0, &digest, &tick.signature) {
            eprintln!("Signature of Computor.({}) is not correct\n", tick.computor_index);
            return Ok(false);
        }
    }
        //All Ticks Verified
        let mut passed: bool = false;
        let mut vote_indices: Vec<Vec<i32>> = Vec::new();
        let mut unique_votes: Vec<Tick> = Vec::new();
        get_unique_votes(ticks, &mut unique_votes, &mut vote_indices);
        //println!("Number of unique votes: {}", unique_votes.len());
        for (index, _) in unique_votes.iter().enumerate() {
            if vote_indices[index].len() >= 451 {
                passed = true;
            } else {
                passed = false;
            }
            //println!("Vote #{} (voted by {} computors ID) ", index, vote_indices[index].len());
        }
    Ok(passed)
}

#[inline]
fn compare_votes(a: &Tick, b: &Tick) -> bool {
    a.epoch == b.epoch &&
            a.year == b.year &&
            a.month == b.month &&
            a.day == b.day &&
            a.hour == b.hour &&
            a.minute == b.minute &&
            a.second == b.second &&
            a.millisecond == b.millisecond &&
            a.prev_resource_testing_digest == b.prev_resource_testing_digest &&
            a.prev_spectrum_digest.iter().zip(b.prev_spectrum_digest.iter()).all(|(a,b)| a == b) &&
            a.prev_universe_digest.iter().zip(b.prev_universe_digest.iter()).all(|(a,b)| a == b) &&
            a.prev_computer_digest.iter().zip(b.prev_computer_digest.iter()).all(|(a,b)| a == b) &&
            a.prev_transaction_body_digest == b.prev_transaction_body_digest &&
            a.transaction_digest.iter().zip(b.transaction_digest.iter()).all(|(a,b)| a == b) &&
            a.expected_next_tick_transaction_digest.iter().zip(b.expected_next_tick_transaction_digest.iter()).all(|(a,b)| a == b)
}

fn get_unique_votes(votes: &Vec<Tick>, unique_votes: &mut Vec<Tick>, vote_indices: &mut Vec<Vec<i32>>) {
    if votes.len() == 0 {
        return;
    }
    unique_votes.push(votes[0].clone());
    vote_indices.resize(1, Vec::new());
    vote_indices[0].push(votes[0].computor_index as i32);
    for (i, el) in votes.iter().enumerate() {
        if i == 0 {
            continue;
        }
        let mut vote_index: i32 = -1;
        for (j, _) in unique_votes.iter().enumerate() {
            if compare_votes(&votes[i], &unique_votes[j]) {
                vote_index = j as i32;
                break;
            }
        }
        if vote_index != -1 {
            vote_indices[vote_index as usize].push(votes[i].computor_index as i32);
        } else {
            unique_votes.push(votes[i].clone());
            vote_indices.resize(vote_indices.len() + 1, Vec::new());
            let M = vote_indices.len() - 1;
            vote_indices[M].push(votes[i].computor_index as i32);
        }
    }
}

#[test]
fn test_get_quorum_votes() {

    let pub_key: [u8; 32] = [
        0x57, 0xB2, 0xAE, 0xCF, 0x5D, 0x2B, 0xFD, 0x15, 0xCE, 0x24, 0x3D, 0xA4, 0x85, 0x24, 0x58, 0x31,
        0x6C, 0x5A, 0xE6, 0x72, 0x12, 0xCC, 0x50, 0x25, 0xFB, 0x40, 0x70, 0xEE, 0xBE, 0xEE, 0x5B, 0x12];

    let signature: [u8; 64] = [
        0xB9, 0x69, 0x49, 0x9C, 0x5F, 0xEE, 0xB5, 0x9B,
        0xCD, 0xE6, 0x8D, 0x3D, 0x81, 0xB1, 0x0B, 0x60,
        0xA0, 0x46, 0xE6, 0xFF, 0x1B, 0x5B, 0x90, 0xF6,
        0xF4, 0x10, 0x74, 0x3B, 0x98, 0xFF, 0x18, 0x35,
        0xA8, 0x91, 0x40, 0xB4, 0xA8, 0x73, 0xDB, 0x9B,
        0x0A, 0xAC, 0x8F, 0xBE, 0xEC, 0x49, 0x08, 0xDA,
        0xB1, 0xD9, 0x23, 0x06, 0x26, 0x7D, 0x3B, 0xB7,
        0xC8, 0x1C, 0x32, 0xD0, 0xEF, 0x0B, 0x23, 0x00
    ];

    let mut data: [u8; size_of::<BroadcastComputors>()] = [0; size_of::<BroadcastComputors>()];
    let epoch: u16 = 163;
    data[0..2].copy_from_slice(epoch.to_le_bytes().as_slice());
    let mut index = 2;
    for i in 0..NUMBER_COMPUTORS {
        data[i*32 + 2..i*32+2 + 32].copy_from_slice(&pub_key);
    }
    data[2 + 32 * NUMBER_COMPUTORS..].copy_from_slice(&signature);
    
    let bc: BroadcastComputors = BroadcastComputors::new(&data);
    
    let data = vec![
        179, 1, 162, 0, 78, 218, 145, 1, 0, 0, 55, 43, 1, 27, 5, 25, 182, 148, 180, 150,
        76, 154, 174, 13, 16, 201, 116, 192, 182, 141, 129, 61, 46, 136, 177, 190, 7, 255,
        107, 15, 233, 183, 56, 140, 9, 242, 110, 133, 173, 26, 125, 241, 64, 181, 236, 55,
        53, 192, 108, 141, 20, 198, 42, 104, 198, 1, 132, 246, 45, 82, 35, 205, 214, 62, 35,
        56, 1, 234, 106, 113, 28, 219, 120, 220, 33, 228, 17, 16, 13, 104, 51, 204, 84, 140,
        159, 233, 35, 143, 84, 38, 162, 102, 229, 91, 82, 50, 161, 198, 72, 143, 183, 99,
        12, 139, 19, 77, 144, 157, 103, 28, 151, 234, 24, 108, 5, 149, 224, 248, 209, 84, 154,
        135, 37, 24, 156, 151, 121, 239, 217, 233, 227, 236, 3, 55, 134, 252, 6, 161, 188, 127,
        161, 203, 249, 4, 216, 69, 216, 224, 239, 36, 200, 119, 196, 123, 107, 129, 17, 33,
        78, 46, 196, 109, 51, 94, 138, 198, 108, 18, 210, 47, 153, 184, 172, 250, 202, 217, 176,
        44, 37, 250, 70, 81, 20, 58, 197, 174, 255, 27, 237, 23, 224, 145, 6, 58, 88, 180, 142,
        140, 143, 106, 151, 252, 237, 56, 218, 227, 85, 232, 191, 249, 119, 219, 106, 139, 178,
        76, 66, 49, 196, 24, 48, 86, 28, 24, 146, 197, 167, 85, 86, 70, 144, 53, 83, 54, 122,
        198, 43, 11, 82, 113, 203, 18, 95, 6, 89, 139, 90, 24, 65, 77, 211, 194, 17, 217, 170,
        221, 6, 224, 141, 205, 105, 41, 49, 91, 247, 209, 181, 212, 239, 220, 177, 78, 162, 18,
        153, 203, 141, 98, 0, 2, 182, 209, 242, 147, 212, 202, 202, 49, 88, 208, 111, 41, 217,
        14, 33, 231, 49, 128, 142, 189, 210, 240, 97, 68, 72, 91, 252, 169, 211, 223, 227, 213,
        39, 253, 104, 79, 159, 15, 221, 136, 206, 123, 250, 21, 74, 165, 238, 222, 77, 243, 203,
        118, 96, 203, 34, 9, 207, 253, 230, 127, 25, 0
    ];
    let t: Tick = Tick::new(&data);
    let mut v: Vec<Tick> = Vec::with_capacity(1);
    v.push(t);
    let result = get_quorum_votes(&bc, &v).unwrap();
    println!("{}", result);
}