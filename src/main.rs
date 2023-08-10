fn main() {
    #![feature(stmt_expr_attributes)]
    /*  Full Key Generation (Randomly Generated Secret) */
    //let mut generated_secret_key: [u8; 32] = [0; 32];
    //let mut generated_public_key: [u8; 64] = [0; 64];
    //let full_key_ret_val = unsafe { SchnorrQ_FullKeyGeneration(generated_secret_key.as_mut_ptr(), generated_public_key.as_mut_ptr() ) };
    //println!("SchnorrQ_FullKeyGeneration returned: {:?}", full_key_ret_val);
    //println!("SchnorrQ_FullKeyGeneration Secret Key: (Len={:?}) {:?}", generated_secret_key.len(), generated_secret_key);
    //println!("SchnorrQ_FullKeyGeneration Public Key: (Len={:?}) {:?}", generated_public_key.len(), generated_public_key);

/*
    let val = identity(test_seed, 0).unwrap();
    let id = std::str::from_utf8(val.as_slice()).unwrap();
    println!("Seed=({})", &test_seed);
    println!("identity returned (Len={}) - {:?}", val.len(), id);
*/
/*
    let test_seed = "lcehvbvddggkjfnokduyjuiyvkklrvrmsaozwbvjlzvgvfipqpnkkuf";
    let id = identity(test_seed, 0);
    match identity_to_address(&id) {
        Ok(address) => {
            println!("Generated Address: {}", address);
        },
        Err(err) => println!("Error Generating Address! : {}", err)
    }
*/
    println!("Hello, World!");

}
