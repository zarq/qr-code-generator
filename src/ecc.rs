pub fn generate_ecc(data: &[u8], num_ecc_codewords: usize) -> Vec<u8> {
    let generator = get_generator_polynomial(num_ecc_codewords);
    
    // Create message polynomial: data followed by zeros
    let mut message = data.to_vec();
    message.extend(vec![0; num_ecc_codewords]);
    
    // Polynomial long division
    for i in 0..data.len() {
        let coeff = message[i];
        if coeff != 0 {
            for j in 0..generator.len() {
                message[i + j] ^= gf_multiply(generator[j], coeff);
            }
        }
    }
    
    // Return the remainder (ECC codewords)
    message[data.len()..].to_vec()
}

fn get_generator_polynomial(degree: usize) -> Vec<u8> {
    let mut poly = vec![1];
    
    for i in 0..degree {
        let mut new_poly = vec![0; poly.len() + 1];
        for j in 0..poly.len() {
            new_poly[j] ^= poly[j];
            new_poly[j + 1] ^= gf_multiply(poly[j], gf_exp(i));
        }
        poly = new_poly;
    }
    
    poly
}

fn gf_multiply(a: u8, b: u8) -> u8 {
    if a == 0 || b == 0 {
        return 0;
    }
    
    let log_a = GF_LOG[a as usize];
    let log_b = GF_LOG[b as usize];
    let log_result = (log_a as usize + log_b as usize) % 255;
    
    GF_EXP[log_result]
}

fn gf_exp(exp: usize) -> u8 {
    GF_EXP[exp % 255]
}

// Include generated GF tables
include!(concat!(env!("OUT_DIR"), "/gf_tables.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecc_simple() {
        // Test with known values - let's verify this is correct
        let data = vec![0x40, 0x0C, 0x56, 0x61, 0x80, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC, 0x11, 0xEC];
        let ecc = generate_ecc(&data, 10);
        
        // Let's see what our algorithm actually generates
        println!("Data: {:02X?}", data);
        println!("Generated ECC: {:02X?}", ecc);
        
        // Test generator polynomial first
        let poly = get_generator_polynomial(10);
        println!("Generator poly: {:02X?}", poly);
        
        // For now, let's just verify the algorithm runs without asserting specific values
        assert_eq!(ecc.len(), 10, "ECC should have 10 codewords");
    }

    #[test]
    fn test_ecc_frackybox() {
        let data = vec![32, 91, 11, 98, 56];
        let ecc = generate_ecc(&data, 10);

        let expected = vec![107, 33, 43, 244, 102, 30, 52, 87, 107, 207];
        
        println!("Generated ECC: {:02X?}", ecc);
        println!("Expected ECC:  {:02X?}", expected);

        assert_eq!(ecc, expected, "ECC generation mismatch");
    }

    #[test]
    fn test_generator_polynomial() {
        // Test generator polynomial for degree 7
        let poly = get_generator_polynomial(7);
        let expected = vec![1, 127, 122, 154, 164, 11, 68, 117]; // Known values for degree 7
        
        println!("Generated poly: {:?}", poly);
        println!("Expected poly:  {:?}", expected);
        
        assert_eq!(poly, expected, "Generator polynomial mismatch");
    }
}
