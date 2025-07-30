use crypto::qubic_identities::{get_public_key_from_identity, verify};
use crate::{ARBITRATOR, NUMBER_COMPUTORS, TICK_TYPE};
use crate::computor::BroadcastComputors;
use crate::tick::Tick;

pub fn get_quorum_votes(bc: &BroadcastComputors, ticks: &Vec<Tick>) -> Result<bool, String> {
    const NUM_VOTE_FLAGS: usize = (NUMBER_COMPUTORS + 7) / 8;
    let _arbitrator: [u8; 32] = get_public_key_from_identity(&String::from(ARBITRATOR)).unwrap();
    let mut _vote_flags: [u8; NUM_VOTE_FLAGS] = [0; NUM_VOTE_FLAGS];
    if ticks.len() == 0 as usize {
        return Ok(false);
    }
    //println!("Getting Quorum Votes For Tick {}", ticks.first().unwrap().tick);
    for (_, vote) in ticks.iter().enumerate() {
        let mut tick = vote.clone();
        tick.computor_index = tick.computor_index ^ TICK_TYPE as u16;
        let digest: [u8; 32] = tick.hash();
        tick.computor_index = tick.computor_index ^ TICK_TYPE as u16;
        let pub_key = &bc.pub_keys[tick.computor_index as usize];
        if !verify(&pub_key, &digest, &tick.signature) {
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
    //println!("Quorum Passed: {:?}", passed);
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
    for (i, _) in votes.iter().enumerate() {
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
            let m: usize = vote_indices.len() - 1;
            vote_indices[m].push(votes[i].computor_index as i32);
        }
    }
}

mod test_quorum_votes {
    #![allow(dead_code, unused)]
    use crate::computor::BroadcastComputors;
    use crate::consensus_tests::epoch_163_computors;
    use crate::quorum_votes::get_quorum_votes;
    use crate::tick::Tick;

    #[test]
    fn test_get_quorum_votes() {
        let bc: BroadcastComputors = BroadcastComputors::new(epoch_163_computors());

        let data = vec![224, 1, 163, 0, 64, 238, 147, 1, 1, 0, 15, 50, 1, 29, 5,
                        25, 206, 152, 199, 234, 61, 21, 102, 58, 223, 10, 105, 47, 208, 8,
                        154, 93, 83, 76, 203, 213, 9, 40, 168, 225, 147, 205, 92, 42, 135,
                        204, 191, 67, 252, 43, 165, 74, 36, 218, 29, 64, 143, 93, 15, 70,
                        190, 25, 200, 189, 170, 69, 44, 70, 109, 125, 11, 129, 168, 18, 42,
                        124, 9, 194, 40, 167, 209, 110, 177, 122, 123, 200, 250, 41, 114, 43,
                        180, 215, 7, 183, 17, 142, 198, 123, 58, 132, 0, 254, 192, 137, 172,
                        21, 237, 134, 98, 198, 215, 176, 187, 83, 53, 69, 238, 140, 219, 125,
                        192, 133, 58, 181, 191, 200, 126, 238, 77, 220, 71, 117, 26, 213, 54,
                        85, 48, 177, 115, 57, 203, 185, 13, 116, 16, 253, 253, 153, 82, 14, 18,
                        138, 199, 50, 74, 57, 29, 216, 227, 178, 126, 208, 223, 40, 47, 26, 52,
                        120, 149, 251, 204, 134, 64, 61, 44, 63, 161, 126, 124, 200, 217, 154,
                        144, 254, 30, 242, 55, 114, 86, 233, 92, 192, 193, 165, 218, 188, 133,
                        246, 163, 56, 212, 191, 77, 110, 176, 153, 121, 15, 189, 0, 62, 103, 230,
                        49, 82, 255, 246, 12, 252, 67, 235, 120, 41, 232, 37, 195, 38, 96, 98, 203,
                        79, 70, 186, 11, 170, 207, 50, 194, 155, 89, 12, 182, 147, 223, 100, 108,
                        241, 234, 1, 181, 15, 87, 7, 33, 252, 128, 214, 164, 49, 160, 88, 228, 11,
                        228, 48, 194, 88, 232, 79, 86, 163, 186, 132, 17, 127, 101, 128, 8, 254, 30,
                        103, 248, 103, 110, 110, 31, 3, 143, 0, 0, 115, 132, 161, 68, 94, 125, 186,
                        27, 252, 240, 162, 62, 61, 87, 147, 168, 69, 82, 62, 208, 253, 166, 162, 184,
                        96, 174, 48, 81, 81, 121, 49, 75, 71, 58, 80, 248, 217, 123, 219, 37, 203,
                        226, 32, 7, 43, 174, 90, 41, 163, 27, 137, 139, 254, 64, 135, 56, 117, 75,
                        147, 118, 31, 0];
        let t: Tick = Tick::new(&data);
        let mut v: Vec<Tick> = Vec::with_capacity(1);
        v.push(t);
        let result = get_quorum_votes(&bc, &v).unwrap();
        assert_eq!(result, false);
    }
}